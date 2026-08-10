import { ButtonHTMLAttributes, forwardRef, memo } from "react";
import { cn } from "@/lib/utils";
import { Loader2 } from "lucide-react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "gold" | "outline" | "danger" | "ghost" | "link" | "success";
  size?: "sm" | "md" | "lg";
  loading?: boolean;
  icon?: React.ReactNode;
}

const Button = memo(forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "primary", size = "md", loading, icon, children, disabled, ...props }, ref) => {
    const variants = {
      primary: "btn-primary",
      gold: "btn-gold",
      outline: "btn-outline",
      danger: "btn-danger",
      success: "bg-emerald-600 hover:bg-emerald-700 text-pure-white",
      ghost: "btn-ghost",
      link: "text-brand-400 hover:text-brand-300 underline underline-offset-2",
    };
    const sizes = {
      sm: "px-3 py-1.5 text-xs rounded-lg",
      md: "px-5 py-2.5 text-sm rounded-xl",
      lg: "px-7 py-3 text-base rounded-xl",
    };
    return (
      <button
        ref={ref}
        className={cn(variants[variant], sizes[size], "inline-flex items-center justify-center gap-2 font-semibold transition-all duration-200", className)}
        disabled={disabled || loading}
        {...props}
      >
        {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : icon}
        {children}
      </button>
    );
  }
));

Button.displayName = "Button";
export default Button;
