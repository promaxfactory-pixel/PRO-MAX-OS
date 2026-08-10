import { cn } from "@/lib/utils";

export default function LoadingSpinner({ size = "md", className }: { size?: "sm" | "md" | "lg"; className?: string }) {
  const sizes = { sm: "w-4 h-4", md: "w-8 h-8", lg: "w-12 h-12" };
  return (
    <div className={cn("flex items-center justify-center", className)}>
      <div className={cn(sizes[size], "border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin")} />
    </div>
  );
}

export function PageLoader() {
  return (
    <div className="flex items-center justify-center h-64">
      <div className="text-center">
        <LoadingSpinner size="lg" />
        <p className="text-sm text-surface-400 mt-4">جاري التحميل...</p>
      </div>
    </div>
  );
}
