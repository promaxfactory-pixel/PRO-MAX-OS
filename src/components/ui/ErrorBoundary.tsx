import React from "react";

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
        <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-red-50 to-orange-50 p-8" dir="rtl">
          <div className="max-w-md text-center">
            <div className="text-6xl mb-4">⚠️</div>
            <h1 className="text-2xl font-bold text-gray-800 mb-2">حدث خطأ غير متوقع</h1>
            <p className="text-gray-600 mb-6">نأسف للإزعاج. يرجى إعادة المحاولة أو الاتصال بالدعم الفني.</p>
            <button
              onClick={() => { this.setState({ hasError: false, error: null }); window.location.href = "/"; }}
              className="px-6 py-3 bg-brand-500 text-white rounded-lg hover:bg-brand-600 transition-colors font-semibold"
            >
              العودة للرئيسية
            </button>
            <details className="mt-4 text-right">
              <summary className="text-sm text-gray-500 cursor-pointer hover:text-gray-700">تفاصيل تقنية</summary>
              <pre className="mt-2 text-xs text-red-700 bg-red-50 p-3 rounded-lg overflow-auto max-h-40">
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
