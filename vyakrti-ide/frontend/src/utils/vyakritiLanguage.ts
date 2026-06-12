import type * as monaco from 'monaco-editor';

export const VYAKRITI_LANG_ID = 'vyakriti';

interface KeywordEntry {
  keyword: string;
  meaning: string;
  description: string;
}

export const VYAKRITI_KEYWORDS: KeywordEntry[] = [
  { keyword: 'मान', meaning: 'var', description: 'Variable declaration.' },
  { keyword: 'कार्य', meaning: 'function', description: 'Function declaration.' },
  { keyword: 'प्रतिफल', meaning: 'return', description: 'Return value from function.' },
  { keyword: 'मुद्रण', meaning: 'print', description: 'Print value to console.' },
  { keyword: 'यदि', meaning: 'if', description: 'Conditional branch.' },
  { keyword: 'तर्हि', meaning: 'then', description: 'Then branch — follows यदि condition.' },
  { keyword: 'अन्यथा', meaning: 'else', description: 'Else branch.' },
  { keyword: 'यावत्', meaning: 'while', description: 'While loop — executes as long as condition is true.' },
  { keyword: 'तावत्', meaning: 'do', description: 'Do-while loop body.' },
  { keyword: 'आयात', meaning: 'import', description: 'Import another module.' },
  { keyword: 'सत', meaning: 'true', description: 'Boolean true literal.' },
  { keyword: 'असत', meaning: 'false', description: 'Boolean false literal.' },
  { keyword: 'च', meaning: 'and', description: 'Logical AND operator.' },
  { keyword: 'वा', meaning: 'or', description: 'Logical OR operator.' },
  { keyword: 'समान', meaning: 'equal', description: 'Equality comparison.' },
  { keyword: 'ऊन', meaning: 'less', description: 'Less-than comparison.' },
  { keyword: 'अग्र', meaning: 'greater', description: 'Greater-than comparison.' },
  { keyword: 'ऊनसमान', meaning: 'less-equal', description: 'Less-than-or-equal comparison.' },
  { keyword: 'अग्रसमान', meaning: 'greater-equal', description: 'Greater-than-or-equal comparison.' },
  { keyword: 'असमान', meaning: 'not-equal', description: 'Not-equal comparison.' },
  { keyword: 'वस्तु_विन्यासः', meaning: 'struct', description: 'Structure/record type declaration.' },
  { keyword: 'गुणधर्म', meaning: 'trait', description: 'Trait/interface declaration.' },
  { keyword: 'अनुष्ठान', meaning: 'impl', description: 'Implementation block for a trait.' },
  { keyword: 'रूपभेदः', meaning: 'enum', description: 'Enumeration type declaration.' },
  { keyword: 'समीक्षा', meaning: 'match', description: 'Pattern matching expression.' },
  { keyword: 'उदात्त', meaning: 'public', description: 'Access modifier — visible everywhere.' },
  { keyword: 'अनुदात्त', meaning: 'private', description: 'Access modifier — visible within current scope.' },
  { keyword: 'स्वरित', meaning: 'protected', description: 'Access modifier — visible in child scopes.' },
];

export function getKeywordMap(): Record<string, KeywordEntry> {
  const map: Record<string, KeywordEntry> = {};
  for (const kw of VYAKRITI_KEYWORDS) {
    map[kw.keyword] = kw;
  }
  return map;
}

// Type keywords (used in type annotations like `मान x : अङ्क = 5`)
const TYPE_KEYWORDS = ['अङ्क', 'सत्यता', 'शब्द', 'दशमलव', 'शून्य'];

const ALL_KEYWORDS = [
  ...VYAKRITI_KEYWORDS.map((kw) => kw.keyword),
  ...TYPE_KEYWORDS,
];

export const vyakritiLanguageDef: monaco.languages.IMonarchLanguage = {
  defaultToken: '',
  tokenPostfix: '.vya',
  keywords: ALL_KEYWORDS,
  typeKeywords: TYPE_KEYWORDS,
  operators: ['=', '>', '<', '!', '==', '<=', '>=', '!=', '+', '-', '*', '/', '%', '&&', '||', '->', '=>', '@'],
  symbols: /[=><!~?:&|+\-*/^%@]+/,
  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
  tokenizer: {
    root: [
      // Devanagari keywords and identifiers
      [/[\u0900-\u097F][\u0900-\u097F0-9_]*/, { cases: { '@keywords': 'keyword', '@default': 'identifier' } }],
      // Latin identifiers (for FFI symbols, variable names, etc.)
      [/[a-zA-Z_][a-zA-Z0-9_]*/, { cases: { '@keywords': 'keyword', '@default': 'identifier' } }],
      { include: '@whitespace' },
      [/[{}()\[\]]/, '@brackets'],
      [/@symbols/, { cases: { '@operators': 'operator', '@default': '' } }],
      [/\d*\.\d+([eE][-+]?\d+)?/, 'number.float'],
      [/\d+/, 'number'],
      [/[;.]/, 'delimiter'],
      [/।/, 'delimiter'],
      [/॥/, 'delimiter'],
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
      [/'[^\\']'/, 'string'],
      [/'/, 'string.invalid'],
    ],
    comment: [
      [/[^/*]+/, 'comment'],
      [/\*\//, 'comment', '@pop'],
      [/[/*]/, 'comment'],
    ],
    string: [
      [/[^\\"]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
    ],
    whitespace: [
      [/[ \t\r\n]+/, 'white'],
      [/\/\*/, 'comment', '@comment'],
      [/\/\/.*$/, 'comment'],
    ],
  },
};

export function registerVyakritiLanguage(monaco: typeof import('monaco-editor')): void {
  const languages = monaco.languages.getLanguages();
  if (languages.some((l) => l.id === VYAKRITI_LANG_ID)) return;

  monaco.languages.register({ id: VYAKRITI_LANG_ID });
  monaco.languages.setMonarchTokensProvider(VYAKRITI_LANG_ID, vyakritiLanguageDef);

  monaco.languages.setLanguageConfiguration(VYAKRITI_LANG_ID, {
    brackets: [['{', '}'], ['[', ']'], ['(', ')']],
    autoClosingPairs: [
      { open: '{', close: '}' }, { open: '[', close: ']' },
      { open: '(', close: ')' }, { open: '"', close: '"' }, { open: "'", close: "'" },
    ],
    surroundingPairs: [
      { open: '{', close: '}' }, { open: '[', close: ']' },
      { open: '(', close: ')' }, { open: '"', close: '"' }, { open: "'", close: "'" },
    ],
    comments: { lineComment: '//', blockComment: ['/*', '*/'] },
    indentationRules: {
      increaseIndentPattern: /\{[^}"']*$/,
      decreaseIndentPattern: /^\s*\}/,
    },
  });

  const kwMap = getKeywordMap();

  // Hover provider — shows keyword meaning in Sanskrit
  monaco.languages.registerHoverProvider(VYAKRITI_LANG_ID, {
    provideHover: (model, position) => {
      const word = model.getWordAtPosition(position);
      if (!word) return null;
      const entry = kwMap[word.word];
      if (!entry) return null;
      return {
        range: {
          startLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endLineNumber: position.lineNumber,
          endColumn: word.endColumn,
        },
        contents: [
          { value: `**${entry.keyword}** — _${entry.meaning}_` },
          { value: entry.description },
        ],
      };
    },
  });

  // Autocomplete / completion provider
  monaco.languages.registerCompletionItemProvider(VYAKRITI_LANG_ID, {
    provideCompletionItems: (_model, _position) => {
      const word = _model.getWordAtPosition(_position);
      const range = word
        ? new monaco.Range(_position.lineNumber, word.startColumn, _position.lineNumber, word.endColumn)
        : new monaco.Range(_position.lineNumber, _position.column, _position.lineNumber, _position.column);

      const keywordItems: monaco.languages.CompletionItem[] = VYAKRITI_KEYWORDS.map((kw) => ({
        label: kw.keyword,
        insertText: kw.keyword,
        documentation: `${kw.description}\n\nMeaning: ${kw.meaning}`,
        kind: monaco.languages.CompletionItemKind.Keyword,
        range,
      }));

      const snippetItems: monaco.languages.CompletionItem[] = [
        {
          label: 'मान (var)',
          insertText: 'मान ${1:नाम} : ${2:प्रकार} = ${3:मूल्य} ।',
          documentation: 'Variable declaration',
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        {
          label: 'कार्य (function)',
          insertText: 'कार्य ${1:नाम}(${2:प्राचल}: ${3:प्रकार}) -> ${4:शून्य} {\n\t$0\n}',
          documentation: 'Function declaration',
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        {
          label: 'यदि (if)',
          insertText: 'यदि (${1:स्थितिः}) तर्हि {\n\t$0\n}',
          documentation: 'Conditional branch',
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        {
          label: 'यावत् (while)',
          insertText: 'यावत् (${1:स्थितिः}) तावत् {\n\t$0\n}',
          documentation: 'While loop',
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        {
          label: 'मुद्रण (print)',
          insertText: 'मुद्रण(${1:"पाठ्यम्"}) ।',
          documentation: 'Print statement',
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        {
          label: 'प्रतिफल (return)',
          insertText: 'प्रतिफल ${1:मूल्यम्} ।',
          documentation: 'Return statement',
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
      ];

      return { suggestions: [...keywordItems, ...snippetItems] };
    },
  });
}
