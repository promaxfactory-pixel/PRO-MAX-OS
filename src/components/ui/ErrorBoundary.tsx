import React from "react";
import i18n from "@/i18n";

interface Props {
  children: React.ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export default class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("ErrorBoundary caught:", error, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-[var(--surface-bg)] p-8" dir="rtl">
          <div className="max-w-md text-center animate-rise-in">
            <div className="text-6xl mb-4" aria-hidden="true">⚠️</div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)] mb-2 font-display">{i18n.t("errorBoundary.title")}</h1>
            <p className="text-[var(--text-secondary)] mb-6">{i18n.t("errorBoundary.message")}</p>
            <button
              onClick={() => { this.setState({ hasError: false, error: null }); window.location.href = "/"; }}
              className="px-6 py-3 bg-brand-500 text-white rounded-xl hover:bg-brand-600 transition-colors font-semibold shadow-lg shadow-brand-500/25"
            >
              {i18n.t("errorBoundary.home")}
            </button>
            <details className="mt-4 text-right">
              <summary className="text-sm text-[var(--text-muted)] cursor-pointer hover:text-[var(--text-secondary)]">{i18n.t("errorBoundary.details")}</summary>
              <pre className="mt-2 text-xs text-red-400 bg-red-500/10 border border-red-500/20 p-3 rounded-lg overflow-auto max-h-40">
                {this.state.error?.message}
              </pre>
            </details>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
