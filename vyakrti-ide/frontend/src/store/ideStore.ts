import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface VirtualFile {
  id: string;
  name: string;
  content: string;
  isDirty: boolean;
}

export interface Diagnostic {
  line: number;
  column: number;
  message: string;
  sanskrit_message?: string;
  severity: 'error' | 'warning' | 'info';
}

export type BuildStatus = 'idle' | 'compiling' | 'running' | 'error' | 'success';
export type TransliterationMode = 'devanagari';
export type ModalType = 'none' | 'create' | 'rename' | 'delete' | 'command-palette';
export type ThemeMode = 'dark' | 'light';

interface WorkspaceFileEntry {
  path: string;
  size: number;
}

const API_BASE = (import.meta.env.VITE_BACKEND_URL || 'http://127.0.0.1:8080').replace(/\/$/, '');
const WS_BASE = API_BASE.replace(/^http:/, 'ws:').replace(/^https:/, 'wss:');

interface IdeState {
  files: VirtualFile[];
  activeFileId: string;
  buildStatus: BuildStatus;
  transliterationMode: TransliterationMode;
  theme: ThemeMode;
  ast: any;
  tokens: string[];
  bytecode: string;
  outputLog: string[];
  diagnostics: Diagnostic[];
  isSettingsOpen: boolean;
  searchQuery: string;
  modalOpen: ModalType;
  modalTargetFileId: string | null;
  wsConnected: boolean;
  abortController: AbortController | null;
  cursorLine: number;
  cursorColumn: number;

  setModal: (type: ModalType, targetId?: string | null) => void;
  setCursorPosition: (line: number, col: number) => void;
  setActiveFile: (id: string) => void;
  updateCurrentContent: (content: string) => void;
  createNewFile: (name: string) => void;
  saveCurrentFile: () => void;
  loadWorkspaceFiles: () => Promise<void>;
  renameFile: (id: string, newName: string) => void;
  deleteFile: (id: string) => void;
  closeFile: (id: string) => void;
  toggleTransliteration: () => void;
  toggleTheme: () => void;
  setSettingsOpen: (open: boolean) => void;
  setSearchQuery: (query: string) => void;
  appendLog: (message: string) => void;
  clearLogs: () => void;
  setBuildStatus: (status: BuildStatus) => void;
  compileAndRun: () => Promise<void>;
  stopExecution: () => void;
  connectWebSocket: () => void;
  setWsConnected: (connected: boolean) => void;
}

const initialContent = `// व्याकृति — संरचना-भाषा
// Vyākṛti — Language of Structure

यदि (सत) तर्हि {
  मुद्रण("नमो व्याकृतिः") ।
}
`;

const OFFLINE_FALLBACK = {
  ast: { type: 'Program' as const, body: [] },
  tokens: [
    'keyword(यदि)        [1:1]',
    'bracket(()           [1:6]',
    'identifier(सत)      [1:7]',
    'bracket())           [1:11]',
    'bracket({)           [1:13]',
    'keyword(मुद्रण)     [2:3]',
    'bracket(()           [2:7]',
    'string("...")        [2:8]',
    'bracket())           [2:23]',
    'delimiter(।)         [2:24]',
    'bracket(})           [3:1]',
  ],
  bytecode:
    '0000  PushConst bool(true)\n0001  JumpIfFalse -> 0012\n0003  PushConst str("नमो व्याकृतिः")\n0004  CallBuiltin "मुद्रण"\n0005  Return',
};

export const useIdeStore = create<IdeState>()(
  persist(
    (set, get) => ({
      files: [{ id: '1', name: 'main.vya', content: initialContent, isDirty: false }],
      activeFileId: '1',
      buildStatus: 'idle',
      transliterationMode: 'devanagari',
      theme: 'dark',
      ast: null,
      tokens: [],
      bytecode: '',
      outputLog: ['व्याकृति-कार्यशाला आरब्धा।'],
      diagnostics: [],
      isSettingsOpen: false,
      searchQuery: '',
      modalOpen: 'none',
      modalTargetFileId: null,
      wsConnected: false,
      abortController: null,
      cursorLine: 1,
      cursorColumn: 1,

      setModal: (type, targetId = null) => set({ modalOpen: type, modalTargetFileId: targetId }),
      setCursorPosition: (line, col) => set({ cursorLine: line, cursorColumn: col }),
      setActiveFile: (id) => set({ activeFileId: id }),
      updateCurrentContent: (content) =>
        set((s) => ({
          files: s.files.map((f) => (f.id === s.activeFileId ? { ...f, content, isDirty: true } : f)),
        })),

      createNewFile: (name) =>
        set((s) => {
          const safeName = name.endsWith('.vya') ? name : `${name}.vya`;
          const id = Date.now().toString();
          return {
            files: [...s.files, { id, name: safeName, content: `// ${safeName}\n// व्याकृतिः\n\n`, isDirty: false }],
            activeFileId: id,
            modalOpen: 'none',
            outputLog: [...s.outputLog, `[व्यवस्था] "${safeName}" निर्मितम्।`],
          };
        }),

      closeFile: (id) =>
        set((s) => {
          const remaining = s.files.filter((f) => f.id !== id);
          if (remaining.length === 0) {
            const freshId = Date.now().toString();
            return { files: [{ id: freshId, name: 'untitled.vya', content: '// व्याकृतिः\n\n', isDirty: false }], activeFileId: freshId };
          }
          const idx = s.files.findIndex((f) => f.id === id);
          const nextId = s.activeFileId === id ? remaining[Math.min(idx, remaining.length - 1)].id : s.activeFileId;
          return { files: remaining, activeFileId: nextId };
        }),

      saveCurrentFile: () =>
        set((s) => {
          const f = s.files.find((x) => x.id === s.activeFileId);
          if (!f) return {};
          fetch(`${API_BASE}/workspace/write`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path: f.name, content: f.content }),
          }).catch(() => undefined);
          return { files: s.files.map((x) => (x.id === s.activeFileId ? { ...x, isDirty: false } : x)), outputLog: [...s.outputLog, `[व्यवस्था] "${f.name}" रक्षितम्।`] };
        }),

      loadWorkspaceFiles: async () => {
        try {
          const listing = await fetch(`${API_BASE}/workspace/list`);
          if (!listing.ok) throw new Error('workspace list failed');
          const data = await listing.json();
          const entries: WorkspaceFileEntry[] = data.files || [];
          const loaded = await Promise.all(entries.slice(0, 40).map(async (entry) => {
            const res = await fetch(`${API_BASE}/workspace/read`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ path: entry.path }),
            });
            const file = await res.json();
            return {
              id: entry.path,
              name: entry.path,
              content: file.content || '',
              isDirty: false,
            } as VirtualFile;
          }));
          if (loaded.length > 0) {
            set((s) => ({
              files: loaded,
              activeFileId: loaded[0].id,
              outputLog: [...s.outputLog, `[व्यवस्था] ${loaded.length} सञ्चिकाः कार्यक्षेत्रात् आनीताः।`],
            }));
          }
        } catch {
          set((s) => ({ outputLog: [...s.outputLog, '[दोषः] कार्यक्षेत्र-सञ्चिकाः आनेतुं न शक्यन्ते।'] }));
        }
      },

      renameFile: (id, newName) =>
        set((s) => {
          const safeName = newName.endsWith('.vya') ? newName : `${newName}.vya`;
          return { files: s.files.map((f) => (f.id === id ? { ...f, name: safeName } : f)), modalOpen: 'none', modalTargetFileId: null, outputLog: [...s.outputLog, `[व्यवस्था] "${safeName}" इति नामकृतम्।`] };
        }),

      deleteFile: (id) =>
        set((s) => {
          const remaining = s.files.filter((f) => f.id !== id);
          const fallback = remaining.length > 0 ? remaining : [{ id: '1', name: 'main.vya', content: '// व्याकृतिः\n\n', isDirty: false }];
          return { files: fallback, activeFileId: s.activeFileId === id ? fallback[0].id : s.activeFileId, modalOpen: 'none', modalTargetFileId: null, outputLog: [...s.outputLog, `[व्यवस्था] सञ्चिका नष्टा।`] };
        }),

      toggleTransliteration: () => set(() => ({ transliterationMode: 'devanagari' })),
      toggleTheme: () => set((s) => ({ theme: s.theme === 'dark' ? 'light' : 'dark' })),
      setSettingsOpen: (open) => set({ isSettingsOpen: open }),
      setSearchQuery: (query) => set({ searchQuery: query }),
      appendLog: (message) => set((s) => ({ outputLog: [...s.outputLog, message] })),
      clearLogs: () => set({ outputLog: [] }),
      setBuildStatus: (status) => set({ buildStatus: status }),

      stopExecution: () => {
        get().abortController?.abort();
        set({ buildStatus: 'idle', abortController: null, outputLog: [...get().outputLog, '[व्यवस्था] उपयोक्त्रा निर्वाहः रद्दीकृतः।'] });
      },

      setWsConnected: (connected) => set({ wsConnected: connected }),

      connectWebSocket: () => {
        try {
          const socket = new WebSocket(`${WS_BASE}/ws`);
          socket.onopen = () => { set({ wsConnected: true }); get().appendLog('[व्यवस्था] वेब-द्वारः संयुक्तः।'); };
          socket.onmessage = (event) => get().appendLog(`[VM] ${event.data}`);
          socket.onclose = () => { set({ wsConnected: false }); };
          socket.onerror = () => { set({ wsConnected: false }); };
        } catch { set({ wsConnected: false }); }
      },

      compileAndRun: async () => {
        const { files, activeFileId } = get();
        const currentFile = files.find((f) => f.id === activeFileId);
        if (!currentFile) { set((s) => ({ outputLog: [...s.outputLog, '[दोषः] न कोऽपि सञ्चिका उद्घाटिता।'] })); return; }

        const controller = new AbortController();
        set({
          buildStatus: 'compiling',
          diagnostics: [],
          outputLog: ['[व्यवस्था] संग्रहणम्...'],
          abortController: controller,
        });

        try {
          const res = await fetch(`${API_BASE}/compile`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ source: currentFile.content }),
            signal: controller.signal,
          });

          if (res.ok) {
            const data = await res.json();
            const hasErrors = data.diagnostics?.some((d: Diagnostic) => d.severity === 'error');
            set({
              ast: data.ast || OFFLINE_FALLBACK.ast,
              tokens: data.tokens || OFFLINE_FALLBACK.tokens,
              bytecode: data.bytecode || OFFLINE_FALLBACK.bytecode,
              diagnostics: data.diagnostics || [],
              abortController: null,
              outputLog: data.output || [],
              buildStatus: hasErrors ? 'error' : 'success',
            });
            return;
          }
        } catch {
          if (get().buildStatus === 'idle') return;
        }

        set({
          ast: OFFLINE_FALLBACK.ast,
          tokens: OFFLINE_FALLBACK.tokens,
          bytecode: OFFLINE_FALLBACK.bytecode,
          diagnostics: [{
            line: 0, column: 0,
            message: "संग्राहकः न प्राप्तः। अनुकृतिः दर्श्यते।",
            severity: 'error',
          }],
          abortController: null,
          outputLog: [
            '── व्याकृतिः पृष्ठभागः न सज्जः ────────────────────',
            '',
            '  [अनुकृतिः — संग्राहकः न संयुक्तः]',
            '',
            `  स्रोतः : ${currentFile.name}`,
            `  पङ्क्तयः : ${currentFile.content.split('\n').length}`,
            `  वर्णाः  : ${currentFile.content.length}`,
            '',
            '────────────────────────────────────────────────────',
            '  संग्राहकम् आरभ्यताम्: cd backend && cargo run',
            '────────────────────────────────────────────────────',
          ],
          buildStatus: 'error',
        });
      },
    }),
    {
      name: 'vyakriti-workspace-storage',
      partialize: (state) => ({
        files: state.files,
        activeFileId: state.activeFileId,
        transliterationMode: state.transliterationMode,
        theme: state.theme,
      }),
    }
  )
);
