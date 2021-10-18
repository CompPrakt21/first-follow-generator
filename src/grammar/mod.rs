use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    iter::FromIterator,
};

mod parse;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Symbol {
    NonTerminal(String),
    Terminal(String),
    Alternate(Vec<Symbol>),
    Sequence(Vec<Symbol>),
    Repeat(Box<Symbol>),
    Optional(Box<Symbol>),
    Epsilon,
}

#[derive(Debug)]
pub struct Grammar {
    pub(crate) rules: HashMap<String, Symbol>,
}

#[derive(Debug, thiserror::Error)]
#[error("Non-terminal '{non_term}' doesn't have a rule in grammar.")]
pub struct IntegrityCheckError {
    non_term: String,
}

enum EpsResult {
    CanBeEps,
    CanNotBeEps,
    EvalError,
}

impl Grammar {
    pub fn parse(s: &str) -> Self {
        parse::parse_grammar(s)
    }

    fn integrity_check_symbol(&self, sym: &Symbol) -> Result<(), IntegrityCheckError> {
        match sym {
            Symbol::NonTerminal(non_term) => {
                if self.rules.contains_key(non_term) {
                    Ok(())
                } else {
                    Err(IntegrityCheckError {
                        non_term: non_term.into(),
                    })
                }
            }
            Symbol::Terminal(_) => Ok(()),
            Symbol::Alternate(symbols) | Symbol::Sequence(symbols) => {
                for sym in symbols {
                    self.integrity_check_symbol(sym)?
                }
                Ok(())
            }
            Symbol::Repeat(s) | Symbol::Optional(s) => self.integrity_check_symbol(s),
            Symbol::Epsilon => Ok(()),
        }
    }

    pub fn integrity_check(&self) -> Result<(), IntegrityCheckError> {
        for sym in self.rules.values() {
            self.integrity_check_symbol(sym)?;
        }
        Ok(())
    }

    pub fn can_produce_eps(&self, sym: &Symbol) -> bool {
        let mut visited = HashSet::new();
        match self.can_produce_eps_rec(sym, &mut visited) {
            EpsResult::CanBeEps => true,
            EpsResult::CanNotBeEps => false,
            EpsResult::EvalError => panic!("Couldn't evaluate if it can eps."),
        }
    }

    fn can_produce_eps_rec(&self, sym: &Symbol, visited: &mut HashSet<String>) -> EpsResult {
        match sym {
            Symbol::NonTerminal(non_term) => {
                if visited.contains(non_term) {
                    EpsResult::EvalError
                } else {
                    let inserted = visited.insert(non_term.into());

                    let symb = &self.rules[non_term];

                    let res = self.can_produce_eps_rec(symb, visited);

                    if inserted {
                        visited.remove(non_term);
                    }

                    res
                }
            }
            Symbol::Terminal(term) => {
                if term.len() <= 0 {
                    EpsResult::CanBeEps
                } else {
                    EpsResult::CanNotBeEps
                }
            }
            Symbol::Alternate(alts) => {
                let mut error_occurred = false;

                for sym in alts {
                    match self.can_produce_eps_rec(sym, visited) {
                        EpsResult::CanBeEps => return EpsResult::CanBeEps,
                        EpsResult::CanNotBeEps => continue,
                        EpsResult::EvalError => error_occurred = true,
                    }
                }

                if error_occurred {
                    EpsResult::EvalError
                } else {
                    EpsResult::CanNotBeEps
                }
            }
            Symbol::Sequence(seq) => {
                let mut error_occurred = false;

                for sym in seq {
                    match self.can_produce_eps_rec(sym, visited) {
                        EpsResult::CanBeEps => continue,
                        EpsResult::CanNotBeEps => return EpsResult::CanNotBeEps,
                        EpsResult::EvalError => error_occurred = true,
                    }
                }

                if error_occurred {
                    EpsResult::EvalError
                } else {
                    EpsResult::CanNotBeEps
                }
            }
            Symbol::Repeat(_) => EpsResult::CanBeEps,
            Symbol::Optional(_) => EpsResult::CanBeEps,
            Symbol::Epsilon => EpsResult::CanBeEps,
        }
    }

    pub fn calc_first_sets(&self) -> HashMap<String, HashSet<String>> {
        let mut res = self
            .rules
            .keys()
            .map(|non_term| (non_term.to_string(), HashSet::new()))
            .collect();

        loop {
            let mut changed = false;

            for (non_term, symb) in &self.rules {
                let first_set = self.calc_first_set_for_symb(symb, &mut res);

                for term in first_set {
                    changed |= res.get_mut(non_term).unwrap().insert(term);
                }
            }

            if !changed {
                break;
            }
        }

        res
    }

    fn calc_first_set_for_symb(
        &self,
        sym: &Symbol,
        current_first_sets: &mut HashMap<String, HashSet<String>>,
    ) -> HashSet<String> {
        match sym {
            Symbol::NonTerminal(non_term) => current_first_sets
                .entry(non_term.clone())
                .or_insert(HashSet::new())
                .clone(),
            Symbol::Terminal(t) => {
                let mut res = HashSet::new();
                res.insert(t.clone());
                res
            }
            Symbol::Alternate(alts) => {
                let mut res = HashSet::new();

                for child_sym in alts {
                    res.extend(self.calc_first_set_for_symb(child_sym, current_first_sets));
                }

                res
            }
            Symbol::Sequence(seq) => {
                let mut res = HashSet::new();

                for child_sym in seq {
                    res.extend(self.calc_first_set_for_symb(child_sym, current_first_sets));

                    if !self.can_produce_eps(child_sym) {
                        break;
                    }
                }

                res
            }
            Symbol::Repeat(child_sym) | Symbol::Optional(child_sym) => {
                self.calc_first_set_for_symb(child_sym, current_first_sets)
            }
            Symbol::Epsilon => HashSet::new(),
        }
    }

    pub fn calc_follow_sets(
        &self,
        first_sets: &mut HashMap<String, HashSet<String>>,
    ) -> HashMap<String, HashSet<String>> {
        let mut res = HashMap::new();

        loop {
            let mut changed = false;

            for (non_term, symb) in &self.rules {
                changed |= self.update_follow_sets(symb, first_sets, &mut res);

                changed |= sym_follow_is_follow_of_sym(
                    &Symbol::NonTerminal(non_term.clone()),
                    symb,
                    &mut res,
                );
            }

            if !changed {
                break;
            }
        }

        res.into_iter()
            .filter_map(|(sym, follow)| {
                if let Symbol::NonTerminal(non_term) = sym {
                    Some((non_term, follow))
                } else {
                    None
                }
            })
            .collect()
    }

    fn update_follow_sets(
        &self,
        sym: &Symbol,
        first_sets: &mut HashMap<String, HashSet<String>>,
        cur_follow_sets: &mut HashMap<Symbol, HashSet<String>>,
    ) -> bool {
        let mut changed = false;

        match sym {
            Symbol::NonTerminal(_) => {}
            Symbol::Terminal(_) => {}
            Symbol::Alternate(alts) => {
                for child_sym in alts {
                    changed |= self.update_follow_sets(child_sym, first_sets, cur_follow_sets);
                    changed |= sym_follow_is_follow_of_sym(sym, child_sym, cur_follow_sets);
                }
            }
            Symbol::Sequence(seq) => {
                for child_sym in seq {
                    changed |= self.update_follow_sets(child_sym, first_sets, cur_follow_sets);
                }

                for prev_idx in 0..seq.len() - 1 {
                    let sym_prev = &seq[prev_idx];

                    for next_idx in prev_idx + 1..seq.len() {
                        let sym_next = &seq[next_idx];

                        let first_set = self.calc_first_set_for_symb(sym_next, first_sets);

                        for first_term in first_set {
                            changed |= cur_follow_sets
                                .entry(sym_prev.clone())
                                .or_insert(HashSet::new())
                                .insert(first_term.clone());
                        }

                        if !self.can_produce_eps(sym_next) {
                            break;
                        }
                    }
                }

                let last_sym = seq.last().unwrap();

                changed |= sym_follow_is_follow_of_sym(sym, last_sym, cur_follow_sets);
            }
            Symbol::Optional(child_sym) => {
                changed |= self.update_follow_sets(child_sym, first_sets, cur_follow_sets);
                changed |= sym_follow_is_follow_of_sym(sym, child_sym, cur_follow_sets);
            }
            Symbol::Repeat(child_sym) => {
                changed |= self.update_follow_sets(child_sym, first_sets, cur_follow_sets);
                changed |= sym_follow_is_follow_of_sym(sym, child_sym, cur_follow_sets);

                for first in self.calc_first_set_for_symb(sym, first_sets) {
                    changed |= cur_follow_sets
                        .entry(sym.clone())
                        .or_insert(HashSet::new())
                        .insert(first);
                }
            }
            Symbol::Epsilon => {}
        }

        changed
    }

    fn ends_with_non_terms(&self, sym: &Symbol) -> HashSet<String> {
        match sym {
            Symbol::NonTerminal(non_term) => HashSet::from_iter(std::iter::once(non_term.clone())),
            Symbol::Terminal(_) => HashSet::new(),
            Symbol::Alternate(alts) => alts
                .iter()
                .flat_map(|child_sym| self.ends_with_non_terms(child_sym))
                .collect(),
            Symbol::Sequence(seq) => self.ends_with_non_terms(seq.last().unwrap()),
            Symbol::Repeat(child_sym) | Symbol::Optional(child_sym) => {
                self.ends_with_non_terms(child_sym)
            }
            Symbol::Epsilon => HashSet::new(),
        }
    }
}

/// The follow set of sym1 is in follow set of sym2
fn sym_follow_is_follow_of_sym(
    sym1: &Symbol,
    sym2: &Symbol,
    cur_follows: &mut HashMap<Symbol, HashSet<String>>,
) -> bool {
    let followed_terms: Vec<_> = cur_follows
        .entry(sym1.clone())
        .or_insert(HashSet::new())
        .iter()
        .cloned()
        .collect();

    let last_term_follows = cur_follows.entry(sym2.clone()).or_insert(HashSet::new());

    let mut changed = false;

    for follow_term in followed_terms {
        changed |= last_term_follows.insert(follow_term)
    }

    changed
}
