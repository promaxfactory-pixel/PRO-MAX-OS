import { useState, useCallback, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/uiStore";
import { ArrowRight, ArrowLeft, Check, ChevronRight, Sparkles, Keyboard, CheckCircle, X } from "lucide-react";
import { cn } from "@/lib/utils";

interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  illustration: React.ReactNode;
  action?: { label: string; onClick: () => void };
}

const steps: OnboardingStep[] = [
  {
    id: "welcome",
    title: "مرحباً بك في PRO MAX OS",
    description: "نظام تخطيط موارد المؤسسات الأكثر تقدماً، مصمم للسرعة والأناقة والأداء.",
    illustration: (
      <motion.div
        animate={{ rotate: [0, 2, -2, 0] }}
        transition={{ duration: 3, repeat: Infinity }}
        className="w-32 h-32 mx-auto rounded-2xl bg-gradient-to-br from-brand-500/20 to-brand-500/5 flex items-center justify-center"
      >
        <Sparkles className="w-16 h-16 text-brand-400" />
      </motion.div>
    ),
  },
  {
    id: "modes",
    title: "7 أوضاع ذكية",
    description: "اختر الوضع الذي يناسب أسلوب عملك: Default، Power، Stability، Focus، Creative، Night، Professional. كل وضع يضبط الألوان والكثافة والتحركات تلقائياً.",
    illustration: (
      <div className="grid grid-cols-3 gap-2 max-w-xs mx-auto">
        {["default", "power", "creative", "night", "professional", "focus", "stability"].map((mode, i) => (
          <motion.div
            key={mode}
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{ delay: i * 0.08 }}
            className="aspect-square rounded-xl border-2 border-surface-700"
            style={{ background: `var(--mode-${mode}-bg, var(--surface-card))` }}
          />
        ))}
      </div>
    ),
  },
  {
    id: "shortcuts",
    title: "اختصارات قوية",
    description: "⌘K للبحث العالمي، ⌘⇧P للوحة الأوامر، ⌘B لطي الشريط، ⌘/ للاختصارات. تعمل في كل مكان.",
    illustration: (
      <div className="space-y-2 max-w-sm mx-auto text-right">
        {[
          ["⌘K", "بحث عام"],
          ["⌘⇧P", "لوحة الأوامر"],
          ["⌘B", "طي الشريط"],
          ["⌘/", "الاختصارات"],
          ["⌘H", "الرئيسية"],
          ["Esc", "إغلاق"],
        ].map(([keys, desc], i) => (
          <motion.div
            key={keys}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: i * 0.06 }}
            className="flex items-center justify-between p-3 rounded-xl bg-surface-800/50 border border-surface-700"
          >
            <span className="text-sm font-medium text-white">{desc}</span>
            <kbd className="px-2 py-1 bg-surface-900 border border-surface-700 rounded-lg text-xs font-mono text-surface-300">{keys}</kbd>
          </motion.div>
        ))}
      </div>
    ),
  },
  {
    id: "navigation",
    title: "تنقل سلس",
    description: "شريط جانبي قابل للطي، تنقل بالمسارات، بحث فوري، ولوحة أوامر للوصول لأي مكان في ثوانٍ.",
    illustration: (
      <motion.div
        animate={{ x: [-20, 20, -20] }}
        transition={{ duration: 2, repeat: Infinity }}
        className="w-64 h-40 mx-auto rounded-xl bg-surface-800/50 border border-surface-700 flex items-center justify-center"
      >
        <ChevronRight className="w-12 h-12 mx-auto text-brand-400/50" />
      </motion.div>
    ),
  },
  {
    id: "ready",
    title: "أنت جاهز للانطلاق",
    description: "استمتع بتجربة ERP مصممة بدقة للأداء والجمال. ابدأ بإنشاء أول فاتورة أو عميل أو منتج.",
    illustration: (
      <motion.div
        animate={{ scale: [1, 1.05, 1] }}
        transition={{ duration: 2, repeat: Infinity }}
        className="w-24 h-24 mx-auto rounded-2xl bg-gradient-to-br from-emerald-500/20 to-emerald-500/5 flex items-center justify-center"
      >
        <CheckCircle className="w-12 h-12 text-emerald-400" />
      </motion.div>
    ),
    action: { label: "ابدأ الآن", onClick: () => {} },
  },
];

export default function Onboarding({ onComplete, onSkip }: { onComplete?: () => void; onSkip?: () => void }) {
  const { t } = useTranslation();
  const { showOnboarding, setShowOnboarding } = useUIStore();
  const [currentStep, setCurrentStep] = useState(0);
  const [direction, setDirection] = useState<"forward" | "backward">("forward");

  useEffect(() => {
    if (!showOnboarding) return;
    const timer = setTimeout(() => {
      if (window.confirm("هل تريد عرض جولة تعريفية سريعة؟")) return;
      setShowOnboarding(false);
      onSkip?.();
    }, 2000);
    return () => clearTimeout(timer);
  }, [showOnboarding, onSkip]);

  if (!showOnboarding) return null;

  const step = steps[currentStep];
  const isLast = currentStep === steps.length - 1;
  const isFirst = currentStep === 0;

  const next = useCallback(() => {
    setDirection("forward");
    if (isLast) {
      setShowOnboarding(false);
      onComplete?.();
    } else {
      setCurrentStep(c => c + 1);
    }
  }, [currentStep, isLast, onComplete]);

  const prev = useCallback(() => {
    setDirection("backward");
    if (!isFirst) setCurrentStep(c => c - 1);
  }, [isFirst]);

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "ArrowRight" || e.key === "Enter") next();
    else if (e.key === "ArrowLeft") prev();
    else if (e.key === "Escape") { setShowOnboarding(false); onSkip?.(); }
  };

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-[200] flex items-center justify-center p-4"
        onClick={onSkip}
      >
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="absolute inset-0 bg-black/70 backdrop-blur-sm"
          onClick={onSkip}
          aria-hidden="true"
        />
        <motion.div
          initial={{ opacity: 0, scale: 0.9, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.9, y: -20 }}
          transition={{ type: "spring", damping: 25, stiffness: 300 }}
          className="relative w-full max-w-2xl mx-4 glass-morphism rounded-3xl overflow-hidden shadow-2xl"
          onClick={e => e.stopPropagation()}
          role="dialog"
          aria-modal="true"
          aria-labelledby="onboarding-title"
        >
          <div className="absolute top-4 left-4 flex gap-2">
            <motion.button
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.95 }}
              onClick={onSkip}
              className="p-2 rounded-xl text-surface-500 hover:text-white hover:bg-white/10 transition-colors"
              aria-label="تخطي"
            >
              <X className="w-5 h-5" />
            </motion.button>
            <motion.div className="hidden sm:flex items-center gap-1 px-3 py-1 bg-surface-900/50 border border-surface-700 rounded-full text-xs text-surface-400">
              <Keyboard className="w-3 h-3" />
              <span>← → للتنقل</span>
            </motion.div>
          </div>

          <div className="p-8">
            <motion.div
              key={step.id}
              initial={{ opacity: 0, y: direction === "forward" ? 30 : -30 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: direction === "forward" ? -30 : 30 }}
              transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
              className="text-center"
            >
              <div className="mb-8">{step.illustration}</div>
              <motion.h2
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.1 }}
                id="onboarding-title"
                className="text-2xl font-bold text-white mb-4"
              >
                {step.title}
              </motion.h2>
              <motion.p
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.2 }}
                className="text-lg text-surface-300 leading-relaxed max-w-xl mx-auto"
              >
                {step.description}
              </motion.p>
            </motion.div>

            <div className="mt-10 flex items-center justify-between">
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={prev}
                disabled={isFirst}
                className={cn(
                  "flex items-center gap-2 px-4 py-3 rounded-xl font-medium transition-all",
                  isFirst
                    ? "text-surface-500 cursor-not-allowed opacity-40"
                    : "text-surface-300 hover:text-white hover:bg-white/5"
                )}
              >
                <ArrowRight className="w-5 h-5" />
                <span>{t("common.back") || "السابق"}</span>
              </motion.button>

              <div className="flex items-center gap-2">
                {steps.map((_, i) => (
                  <motion.button
                    key={i}
                    onClick={() => { setDirection(i > currentStep ? "forward" : "backward"); setCurrentStep(i); }}
                    className="w-2.5 h-2.5 rounded-full transition-all duration-300"
                    animate={{ 
                      scale: i === currentStep ? 1.4 : 1,
                      backgroundColor: i === currentStep ? "var(--mode-accent)" : "var(--surface-500)",
                    }}
                    transition={{ type: "spring", damping: 20, stiffness: 300 }}
                    aria-label={`الخطوة ${i + 1}`}
                    aria-current={i === currentStep ? "step" : undefined}
                  />
                ))}
              </div>

              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={next}
                className={cn(
                  "flex items-center gap-2 px-6 py-3 rounded-xl font-semibold bg-gradient-to-r from-brand-500 to-brand-600 text-white shadow-lg shadow-brand-500/25 hover:shadow-xl hover:shadow-brand-500/40 transition-all",
                  isFirst && "ml-auto"
                )}
              >
                {isFirst ? (
                  <>
                    <span>{t("common.next") || "التالي"}</span>
                    <ArrowLeft className="w-5 h-5" />
                  </>
                ) : isLast ? (
                  <>
                    <span>{step.action?.label || t("common.start") || "ابدأ"}</span>
                    <Check className="w-5 h-5" />
                  </>
                ) : (
                  <>
                    <span>{t("common.next") || "التالي"}</span>
                    <ArrowLeft className="w-5 h-5" />
                  </>
                )}
              </motion.button>
            </div>
          </div>

          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.5 }}
            className="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-3 text-xs text-surface-500"
          >
            <kbd className="px-2 py-1 bg-surface-900 border border-surface-700 rounded-lg font-mono">Esc</kbd>
            <span>{t("onboarding.skip") || "تخطي الجولة"}</span>
          </motion.div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}