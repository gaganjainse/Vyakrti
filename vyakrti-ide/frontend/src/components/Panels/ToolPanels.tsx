import { useState } from 'react';
import { useIdeStore } from '../../store/ideStore';
import { VYAKRITI_KEYWORDS } from '../../utils/vyakritiLanguage';

type TabId = 'ast' | 'tokens' | 'bytecode' | 'grammar';

export const RightPanel = () => {
  const [activeTab, setActiveTab] = useState<TabId>('ast');
  const { ast, tokens, bytecode } = useIdeStore();

  const tabs: { id: TabId; label: string }[] = [
    { id: 'ast', label: 'AST' },
    { id: 'tokens', label: 'टोकन्स्' },
    { id: 'bytecode', label: 'बैटकोड्' },
    { id: 'grammar', label: 'सूची' },
  ];

  return (
    <aside className="w-72 bg-ide-panel flex flex-col border-l border-ide-border">
      <div className="flex border-b border-ide-border text-xs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 py-2 text-center transition-colors ${
              activeTab === tab.id
                ? 'border-b-2 border-ide-accent text-ide-accent bg-ide-surface'
                : 'text-ide-muted hover:text-ide-text'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-auto p-3 font-mono text-[11px]">
        {activeTab === 'ast' && (
          ast ? (
            <pre className="text-ide-success whitespace-pre-wrap">{JSON.stringify(ast, null, 2)}</pre>
          ) : (
            <EmptyState name="AST" />
          )
        )}

        {activeTab === 'tokens' && (
          tokens && tokens.length > 0 ? (
            <div className="space-y-0.5">
              {tokens.map((t, i) => (
                <div key={i} className="text-ide-accent py-0.5 border-b border-ide-border/30 last:border-0">
                  {t}
                </div>
              ))}
            </div>
          ) : (
            <EmptyState name="Tokens" />
          )
        )}

        {activeTab === 'bytecode' && (
          bytecode ? (
            <pre className="text-ide-warning whitespace-pre-wrap">{bytecode}</pre>
          ) : (
            <EmptyState name="Bytecode" />
          )
        )}

        {activeTab === 'grammar' && (
          <div className="space-y-2">
            <div className="text-[10px] text-ide-muted uppercase tracking-wider font-semibold mb-2">
              व्याकृति-शब्दकोशः
            </div>
            {VYAKRITI_KEYWORDS.map((kw, i) => (
              <div key={i} className="border-b border-ide-border/30 pb-1.5 mb-1.5 last:border-0">
                <div className="flex items-center gap-2">
                  <span className="text-ide-accent font-semibold">{kw.keyword}</span>
                </div>
                <div className="text-ide-muted text-[10px] mt-0.5">
                  <span className="text-ide-warning">{kw.meaning}</span> — {kw.description}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </aside>
  );
};

const EmptyState = ({ name }: { name: string }) => (
  <div className="h-full flex items-center justify-center text-ide-muted italic text-xs">
    {name} न जनितः
  </div>
);
