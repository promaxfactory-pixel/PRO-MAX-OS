import { useNavigate } from "react-router-dom";

export default function ForbiddenPage() {
  const navigate = useNavigate();
  return (
    <div className="min-h-screen flex items-center justify-center bg-[var(--surface-bg)] p-8">
      <div className="max-w-md text-center">
        <div className="text-8xl mb-6 font-bold bg-gradient-to-br from-red-400 to-orange-400 bg-clip-text text-transparent">403</div>
        <h1 className="text-2xl font-bold text-[var(--text-primary)] mb-2">غير مصرح بالوصول</h1>
        <p className="text-[var(--text-secondary)] mb-2">ليس لديك الصلاحيات الكافية للوصول إلى هذه الصفحة.</p>
        <p className="text-[var(--text-tertiary)] text-sm mb-8">يرجى التواصل مع المشرف إذا كنت تعتقد أن هذا خطأ.</p>
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
