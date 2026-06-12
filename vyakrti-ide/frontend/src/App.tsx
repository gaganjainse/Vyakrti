import { ErrorBoundary } from './components/Panels/ErrorBoundary';
import { TopBar } from './components/Toolbar/TopBar';
import { ProjectExplorer } from './components/Sidebar/ProjectExplorer';
import { EditorTabs } from './components/Editor/EditorTabs';
import { CodeEditor } from './components/Editor/CodeEditor';
import { CommandPalette } from './components/Modals/CommandPalette';
import { WorkspaceModals } from './components/Modals/WorkspaceModals';
import { SettingsPanel } from './components/Panels/SettingsPanel';
import { RightPanel } from './components/Panels/ToolPanels';
import { DiagnosticsConsole } from './components/Panels/DiagnosticsConsole';
import { StatusBar } from './components/Panels/StatusBar';
import { useIdeStore } from './store/ideStore';
import { useEffect } from 'react';

export default function App() {
  const { isSettingsOpen, setSettingsOpen, theme } = useIdeStore();

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
  }, [theme]);

  return (
    <ErrorBoundary>
      <div className="h-screen w-screen bg-ide-bg text-ide-text flex flex-col font-sans overflow-hidden">
        <TopBar />

        <div className="flex flex-1 overflow-hidden">
          <ErrorBoundary>
            <ProjectExplorer />
          </ErrorBoundary>

          <main className="flex-1 flex flex-col min-w-0 relative bg-ide-panel">
            <EditorTabs />
            <div className="flex-1 relative">
              <ErrorBoundary>
                <CodeEditor />
              </ErrorBoundary>
            </div>
          </main>

          <ErrorBoundary>
            <RightPanel />
          </ErrorBoundary>
        </div>

        <ErrorBoundary>
          <DiagnosticsConsole />
        </ErrorBoundary>

        <ErrorBoundary>
          <StatusBar />
        </ErrorBoundary>

        <ErrorBoundary><WorkspaceModals /></ErrorBoundary>
        <ErrorBoundary><CommandPalette /></ErrorBoundary>

        {isSettingsOpen && <SettingsPanel onClose={() => setSettingsOpen(false)} />}
      </div>
    </ErrorBoundary>
  );
}
