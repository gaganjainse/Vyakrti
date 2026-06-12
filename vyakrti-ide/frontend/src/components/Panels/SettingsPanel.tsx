import { X, Sun, Moon, Languages, Wifi, WifiOff } from 'lucide-react';
import { useIdeStore } from '../../store/ideStore';

interface SettingsPanelProps {
  onClose: () => void;
}

export const SettingsPanel = ({ onClose }: SettingsPanelProps) => {
  const {
    transliterationMode, toggleTransliteration, theme, toggleTheme,
    wsConnected, connectWebSocket, files,
  } = useIdeStore();

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--ide-overlay)] backdrop-blur-sm select-none p-4">
      <div className="absolute inset-0" onClick={onClose} />

      <div className="bg-ide-panel border border-ide-border rounded-lg shadow-2xl w-full max-w-lg overflow-hidden relative z-10 animate-fade-in">
        <div className="flex items-center justify-between border-b border-ide-border px-4 py-3 bg-ide-surface">
          <span className="text-sm font-semibold text-ide-text">समायोजनम्</span>
          <button onClick={onClose} className="toolbar-btn">
            <X size={14} />
          </button>
        </div>

        <div className="p-4 space-y-5 max-h-[70vh] overflow-y-auto">
          <Section title="संग्राहकः">
            <SettingRow label="पृष्ठभाग-URL" help="यत्र व्याकृतिः संग्राहकः चलति">
              <input
                type="text"
                readOnly
                value="http://127.0.0.1:8080/compile"
                className="ide-input flex-1 text-[11px]"
              />
            </SettingRow>
            <SettingRow label="वेब-द्वारः" help="प्रत्यक्ष-यन्त्रसम्बन्धः">
              <div className="flex items-center gap-2">
                <span className={`flex items-center gap-1 text-xs ${wsConnected ? 'text-ide-success' : 'text-ide-muted'}`}>
                  {wsConnected ? <><Wifi size={12} /> संयुक्तः</> : <><WifiOff size={12} /> विच्छिन्नः</>}
                </span>
                {!wsConnected && (
                  <button onClick={connectWebSocket} className="px-2 py-1 text-[10px] bg-ide-accent text-[#181825] rounded font-medium hover:bg-blue-400">
                    संयोजयतु
                  </button>
                )}
              </div>
            </SettingRow>
          </Section>

          <Section title="सम्पादकः">
            <SettingRow label="लिप्यन्तरम्" help="देवनागरी-लिपिः">
              <button
                onClick={toggleTransliteration}
                className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs bg-ide-surface text-ide-text rounded border border-ide-border hover:bg-ide-hover transition-colors"
              >
                <Languages size={12} className="text-ide-accent" />
                {transliterationMode === 'devanagari' ? 'देवनागरी' : 'SLP1'}
              </button>
            </SettingRow>
            <SettingRow label="रूपम्" help="श्यामं वा प्रकाशं वा">
              <button
                onClick={toggleTheme}
                className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs bg-ide-surface text-ide-text rounded border border-ide-border hover:bg-ide-hover transition-colors"
              >
                {theme === 'dark' ? <Sun size={12} /> : <Moon size={12} />}
                {theme === 'dark' ? 'Light Mode' : 'Dark Mode'}
              </button>
            </SettingRow>
          </Section>

          <Section title="कार्यक्षेत्रम्">
            <SettingRow label="उद्घाटिताः" help="सक्रियाः सञ्चिकाः">
              <span className="text-xs text-ide-muted">{files.length} सञ्चिका(ः)</span>
            </SettingRow>
            <div className="text-[11px] text-ide-muted mt-2">
              सञ्चिकाः स्वचलितरूपेण धार्यन्ते।
            </div>
          </Section>

          <Section title="कुञ्जी-मार्गाः">
            <ShortcutRow keys="Ctrl+N" label="नूतना सञ्चिका" />
            <ShortcutRow keys="Ctrl+S" label="सञ्चिकां रक्षतु" />
            <ShortcutRow keys="Ctrl+Enter" label="संगृह्णातु च चालयतु" />
            <ShortcutRow keys="Ctrl+P" label="आज्ञा-फलकम्" />
            <ShortcutRow keys="Escape" label="संवादान् पिदधातु" />
          </Section>
        </div>
      </div>
    </div>
  );
};

const Section = ({ title, children }: { title: string; children: React.ReactNode }) => (
  <div>
    <h3 className="text-xs font-semibold text-ide-muted uppercase tracking-wider mb-2">{title}</h3>
    <div className="space-y-2">{children}</div>
  </div>
);

const SettingRow = ({
  label, help, children,
}: {
  label: string;
  help?: string;
  children: React.ReactNode;
}) => (
  <div className="flex items-center justify-between gap-4">
    <div>
      <span className="text-sm text-ide-text">{label}</span>
      {help && <p className="text-[10px] text-ide-muted">{help}</p>}
    </div>
    {children}
  </div>
);

const ShortcutRow = ({ keys, label }: { keys: string; label: string }) => (
  <div className="flex items-center justify-between py-1">
    <span className="text-xs text-ide-muted">{label}</span>
    <kbd className="text-[10px] font-mono bg-ide-surface border border-ide-border rounded px-1.5 py-0.5 text-ide-muted">
      {keys}
    </kbd>
  </div>
);
