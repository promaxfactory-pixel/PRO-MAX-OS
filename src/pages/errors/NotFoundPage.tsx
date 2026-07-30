import { useNavigate } from "react-router-dom";

export default function NotFoundPage() {
  const navigate = useNavigate();
  return (
    <div className="min-h-screen flex items-center justify-center bg-[var(--surface-bg)] p-8">
      <div className="max-w-md text-center">
        <div className="text-8xl mb-6 font-bold bg-gradient-to-br from-brand-400 to-gold-400 bg-clip-text text-transparent">404</div>
        <h1 className="text-2xl font-bold text-[var(--text-primary)] mb-2">الصفحة غير موجودة</h1>
        <p className="text-[var(--text-secondary)] mb-8">الصفحة التي تبحث عنها غير موجودة أو تم نقلها.</p>
        <button
          onClick={() => navigate("/")}
          className="px-8 py-3 bg-brand-500 text-pure-white rounded-xl hover:bg-brand-600 transition-colors font-semibold shadow-lg shadow-brand-500/20"
        >
          العودة للرئيسية
        </button>
      </div>
    </div>
  );
}
