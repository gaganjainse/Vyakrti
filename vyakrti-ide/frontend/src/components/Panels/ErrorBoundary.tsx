import { Component, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        this.props.fallback || (
          <div className="flex items-center justify-center h-full bg-ide-surface text-ide-error p-4">
            <div className="text-center">
              <p className="text-sm font-semibold mb-1">चित्रीकरण-दोषः</p>
              <p className="text-xs opacity-70 font-mono">{this.state.error?.message}</p>
            </div>
          </div>
        )
      );
    }
    return this.props.children;
  }
}
