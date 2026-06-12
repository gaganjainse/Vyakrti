import { useState } from 'react';
import { FileText, Folder, ChevronRight, ChevronDown, Edit2, Trash2, Plus, File, SearchCode } from 'lucide-react';
import { useIdeStore } from '../../store/ideStore';

export const ProjectExplorer = () => {
  const { files, activeFileId, setActiveFile, setModal, searchQuery } = useIdeStore();
  const [collapsed, setCollapsed] = useState(false);

  const filteredFiles = files.filter((file) => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return true;
    return file.name.toLowerCase().includes(q) || file.content.toLowerCase().includes(q);
  });

  return (
    <aside className="w-56 border-r border-ide-border bg-ide-panel flex flex-col select-none">
      <div className="panel-header">
        <Folder size={13} />
        <span>अन्वेषकः</span>
      </div>

      <div className="flex-1 overflow-y-auto py-2">
        <div className="flex items-center justify-between px-3 py-1 text-[11px] text-ide-muted font-semibold">
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="flex items-center gap-1 hover:text-ide-text transition-colors"
          >
            {collapsed ? <ChevronRight size={12} /> : <ChevronDown size={12} />}
            <span>कार्यक्षेत्रम्</span>
          </button>
          <button
            onClick={() => setModal('create')}
            className="p-0.5 hover:text-ide-text hover:bg-ide-hover rounded"
            title="नूतना सञ्चिका"
          >
            <Plus size={12} />
          </button>
        </div>

        {!collapsed && (
          <div className="space-y-[1px] mt-1">
            {searchQuery && (
              <div className="px-3 py-1 text-[10px] text-ide-accent flex items-center gap-1">
                <SearchCode size={10} />
                <span>परिणामाः शोधिताः</span>
              </div>
            )}

            {filteredFiles.length === 0 && (
              <div className="px-3 py-6 text-center text-[11px] text-ide-muted italic">
                {searchQuery ? 'न किञ्चित्' : 'सञ्चिका न सन्ति'}
              </div>
            )}

            {filteredFiles.map((file) => {
              const isActive = file.id === activeFileId;
              const ext = file.name.split('.').pop() || 'vya';

              return (
                <div
                  key={file.id}
                  onClick={() => setActiveFile(file.id)}
                  className={`group px-3 pl-7 py-1 cursor-pointer flex items-center justify-between text-xs transition-all ${
                    isActive
                      ? 'bg-ide-active text-ide-accent border-l-2 border-ide-accent'
                      : 'text-ide-muted hover:bg-ide-hover hover:text-ide-text'
                  }`}
                >
                  <div className="flex items-center gap-1.5 overflow-hidden mr-1 min-w-0">
                    {ext === 'vya' || ext === 'vy' ? (
                      <FileText size={12} className={isActive ? 'text-ide-accent' : 'text-ide-muted'} />
                    ) : (
                      <File size={12} className="text-ide-muted" />
                    )}
                    <span className="truncate">{file.name}</span>
                    {file.isDirty && (
                      <span className="w-1 h-1 rounded-full bg-ide-warning flex-shrink-0" />
                    )}
                  </div>

                  <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
                    <button
                      onClick={(e) => { e.stopPropagation(); setModal('rename', file.id); }}
                      className="p-0.5 hover:text-ide-accent hover:bg-ide-hover rounded text-ide-muted"
                      title="नामान्तरम्"
                    >
                      <Edit2 size={10} />
                    </button>
                    <button
                      onClick={(e) => { e.stopPropagation(); setModal('delete', file.id); }}
                      className="p-0.5 hover:text-ide-error hover:bg-ide-hover rounded text-ide-muted"
                      title="नाशयतु"
                    >
                      <Trash2 size={10} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </aside>
  );
};
