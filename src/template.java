package compiler;

import java.util.Map;
import java.util.Set;

public class Grammar \{

	public enum NonTerminals \{
		{{for map in firstmappings}}{map.nonterm}{{if not @last}},
		{{endif}}{{endfor}}
	}

    public static final Map<TokenType, Set<TokenType>> FIRST = Map.ofEntries(
		{{for map in firstmappings}}
        Map.entry(NonTerminals.{map.nonterm}, Set.of({{for term in map.result}}TokenType.{term}{{if not @last}},
		{{endif}}{{endfor}}
	)){{if not @last}},{{endif}}
		{{endfor}}
    );
    public static final Map<TokenType, Set<TokenType>> FOLLOW = Map.ofEntries(
		{{for map in followmappings}}
        Map.entry(NonTerminals.{map.nonterm}, Set.of({{for term in map.result}}TokenType.{term}{{if not @last}},
		{{endif}}{{endfor}}
	)){{if not @last}},{{endif}}
	{{endfor}}
	);
}
