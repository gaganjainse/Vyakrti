import Editor from '@monaco-editor/react';
import { useCallback, useEffect, useRef } from 'react';
import { useIdeStore } from '../../store/ideStore';
import { VYAKRITI_LANG_ID, registerVyakritiLanguage } from '../../utils/vyakritiLanguage';

export const CodeEditor = () => {
  const { files, activeFileId, updateCurrentContent, diagnostics, theme } = useIdeStore();
  const editorRef = useRef<any>(null);
  const monacoRef = useRef<any>(null);

  const activeFile = files.find((f) => f.id === activeFileId);

  const handleEditorWillMount = useCallback((monaco: any) => {
    try {
      registerVyakritiLanguage(monaco);
    } catch (e) {
      console.warn('[Vyākṛti] Language registration failed:', e);
    }
  }, []);

  const handleEditorDidMount = useCallback((editor: any, monaco: any) => {
    editorRef.current = editor;
    monacoRef.current = monaco;

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      useIdeStore.getState().saveCurrentFile();
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      const s = useIdeStore.getState();
      if (s.buildStatus === 'compiling' || s.buildStatus === 'running') {
        s.stopExecution();
      } else {
        s.compileAndRun();
      }
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyN, () => {
      useIdeStore.getState().setModal('create');
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP, () => {
      useIdeStore.getState().setModal('command-palette');
    });

    editor.onDidChangeCursorPosition((e: any) => {
      useIdeStore.getState().setCursorPosition(e.position.lineNumber, e.position.column);
    });
  }, []);

  const handleEditorChange = useCallback(
    (value: string | undefined) => {
      if (value !== undefined) {
        updateCurrentContent(value);
      }
    },
    [updateCurrentContent]
  );

  useEffect(() => {
    if (!editorRef.current || !monacoRef.current) return;

    const editor = editorRef.current;
    const monaco = monacoRef.current;
    const model = editor.getModel();
    if (!model) return;

    if (diagnostics.length > 0) {
      const markers = diagnostics.map((d) => ({
        startLineNumber: d.line || 1,
        startColumn: d.column || 1,
        endLineNumber: d.line || 1,
        endColumn: (d.column || 1) + 8,
        message: d.sanskrit_message
          ? `${d.message}\n[संस्कृतम्] ${d.sanskrit_message}`
          : d.message,
        severity:
          d.severity === 'error'
            ? monaco.MarkerSeverity.Error
            : d.severity === 'warning'
              ? monaco.MarkerSeverity.Warning
              : monaco.MarkerSeverity.Info,
      }));
      monaco.editor.setModelMarkers(model, 'vyakriti-compiler', markers);
    } else {
      monaco.editor.setModelMarkers(model, 'vyakriti-compiler', []);
    }
  }, [diagnostics, activeFileId, files]);

  if (!activeFile) {
    return (
      <div className="h-full flex items-center justify-center text-ide-muted">
        <div className="text-center">
          <p className="text-lg font-semibold mb-1">न कोऽपि सञ्चिका</p>
          <p className="text-sm">.vya सञ्चिकाम् उद्घाटयतु आरम्भाय</p>
        </div>
      </div>
    );
  }

  return (
    <Editor
      height="100%"
      theme={theme === 'dark' ? 'vs-dark' : 'vs'}
      language={VYAKRITI_LANG_ID}
      value={activeFile.content}
      onChange={handleEditorChange}
      beforeMount={handleEditorWillMount}
      onMount={handleEditorDidMount}
      options={{
        minimap: { enabled: false },
        fontSize: 14,
        fontFamily: "'Fira Code', 'Noto Sans Devanagari', Consolas, monospace",
        automaticLayout: true,
        padding: { top: 16 },
        renderLineHighlight: 'all',
        cursorBlinking: 'smooth',
        cursorSmoothCaretAnimation: 'on',
        smoothScrolling: true,
        bracketPairColorization: { enabled: true },
        autoIndent: 'full',
        formatOnPaste: true,
        tabSize: 2,
        wordWrap: 'off',
        lineNumbers: 'on',
        glyphMargin: false,
        folding: true,
        renderWhitespace: 'selection',
        hideCursorInOverviewRuler: true,
        overviewRulerLanes: 0,
        overviewRulerBorder: false,
      }}
    />
  );
};
