package compiler;

import java.util.Map;

public class Grammar \{

	public enum NonT \{
		{{for map in firstmappings}}{map.nonterm}{{if not @last}},
		{{endif}}{{endfor}}

		public TokenSet first() \{
			return FIRST.get(this);
		}

		public TokenSet follow() \{
			return FOLLOW.get(this);
		}

		public boolean firstContains(TokenType type) \{
			return FIRST.get(this).contains(type);
		}

		public boolean followContains(TokenType type) \{
			return FOLLOW.get(this).contains(type);
		}
	}

    public static final Map<NonT, TokenSet> FIRST = Map.ofEntries(
		{{for map in firstmappings}}
        Map.entry(NonT.{map.nonterm}, TokenSet.of({{for term in map.result}}TokenType.{term}{{if not @last}},
		{{endif}}{{endfor}}
	)){{if not @last}},{{endif}}
		{{endfor}}
    );
    public static final Map<NonT, TokenSet> FOLLOW = Map.ofEntries(
		{{for map in followmappings}}
        Map.entry(NonT.{map.nonterm}, TokenSet.of({{for term in map.result}}TokenType.{term}{{if not @last}},
		{{endif}}{{endfor}}
	)){{if not @last}},{{endif}}
	{{endfor}}
	);
}
