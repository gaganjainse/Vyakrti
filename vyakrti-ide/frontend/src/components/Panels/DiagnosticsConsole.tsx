import { useRef, useEffect, useState } from 'react';
import { Terminal, AlertCircle, XCircle, Trash2, Copy, Lightbulb } from 'lucide-react';
import { useIdeStore } from '../../store/ideStore';

interface Explanation {
  explanation: string;
  sanskrit_hint: string;
  suggestion?: string;
}

export const DiagnosticsConsole = () => {
  const { outputLog, diagnostics, clearLogs, appendLog } = useIdeStore();
  const scrollRef = useRef<HTMLDivElement>(null);
  const [explaining, setExplaining] = useState<number | null>(null);
  const [explanations, setExplanations] = useState<Record<number, Explanation>>({});

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [outputLog, diagnostics]);

  const handleCopy = () => {
    const text = [...diagnostics.map((d) => `[${d.severity}] ${d.line}:${d.column} ${d.message}`), ...outputLog].join('\n');
    navigator.clipboard.writeText(text).then(() => {
      appendLog('[व्यवस्था] फलकं प्रतिलिपितम्।');
    }).catch(console.error);
  };

  const handleExplain = async (i: number, message: string) => {
    setExplaining(i);
    try {
      const res = await fetch('http://127.0.0.1:8080/explain', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message }),
      });
      if (res.ok) {
        const data: Explanation = await res.json();
        setExplanations((prev) => ({ ...prev, [i]: data }));
      } else {
        setExplanations((prev) => ({
          ...prev,
          [i]: { explanation: 'पृष्ठभागः न प्राप्तः।', sanskrit_hint: 'स्पष्टीकरणम् न लभ्यते।' },
        }));
      }
    } catch {
      setExplanations((prev) => ({
        ...prev,
          [i]: { explanation: 'पृष्ठभागः अप्राप्यः।', sanskrit_hint: 'पृष्ठभागः न प्राप्तः।' },
      }));
    }
    setExplaining(null);
  };

  return (
    <footer className="h-44 border-t border-ide-border bg-ide-surface flex flex-col flex-shrink-0">
      <div className="panel-header flex-shrink-0">
        <Terminal size={13} />
        <span>फलकम्</span>
        <div className="flex-1" />
        <button onClick={handleCopy} className="p-0.5 hover:text-ide-text hover:bg-ide-hover rounded text-ide-muted" title="सर्वं प्रतिलिप्यताम्">
          <Copy size={11} />
        </button>
        <button onClick={clearLogs} className="p-0.5 hover:text-ide-text hover:bg-ide-hover rounded text-ide-muted" title="फलकं शोध्यताम्">
          <Trash2 size={11} />
        </button>
      </div>

      <div ref={scrollRef} className="flex-1 overflow-y-auto px-3 py-2 font-mono text-xs flex flex-col gap-0.5">
        {diagnostics.length > 0 && (
          <div className="mb-1 border-b border-ide-border/30 pb-1">
            {diagnostics.map((d, i) => (
              <div key={`diag-${i}`} className="mb-1">
                <div
                  className={`flex items-start gap-1.5 py-0.5 ${
                    d.severity === 'error' ? 'text-ide-error' : d.severity === 'warning' ? 'text-ide-warning' : 'text-ide-accent'
                  }`}
                >
                  {d.severity === 'error' ? (
                    <XCircle size={12} className="mt-0.5 shrink-0" />
                  ) : (
                    <AlertCircle size={12} className="mt-0.5 shrink-0" />
                  )}
                  <div className="flex-1">
                    <span className="opacity-70">[{d.line}:{d.column}]</span>{' '}
                    {d.message}
                    {d.sanskrit_message && (
                      <span className="ml-1 opacity-60">| {d.sanskrit_message}</span>
                    )}
                  </div>
                  <button
                    onClick={() => handleExplain(i, d.message)}
                    className="p-0.5 hover:text-ide-accent text-ide-muted shrink-0"
                    title="स्पष्टीकुरुत"
                  >
                    <Lightbulb size={11} />
                  </button>
                </div>
                {explanations[i] && (
                  <div className="ml-5 pl-3 border-l-2 border-ide-accent/30 py-1 text-[11px] space-y-0.5">
                    <div className="text-ide-text">{explanations[i].explanation}</div>
                    <div className="text-ide-accent italic">{explanations[i].sanskrit_hint}</div>
                    {explanations[i].suggestion && (
                      <div className="text-ide-success">💡 {explanations[i].suggestion}</div>
                    )}
                  </div>
                )}
                {explaining === i && (
                  <div className="ml-5 text-ide-muted italic text-[11px]">स्पष्टीकरणम्...</div>
                )}
              </div>
            ))}
          </div>
        )}

        {outputLog.map((log, i) => (
          <div key={`log-${i}`} className="text-ide-text leading-relaxed">
            {log}
          </div>
        ))}

        {outputLog.length === 0 && diagnostics.length === 0 && (
          <div className="text-ide-muted italic">सज्जः</div>
        )}
      </div>
    </footer>
  );
};
