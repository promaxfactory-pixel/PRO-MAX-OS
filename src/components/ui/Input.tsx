import { InputHTMLAttributes, TextareaHTMLAttributes, SelectHTMLAttributes, forwardRef, useId } from "react";
import { cn } from "@/lib/utils";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
  icon?: React.ReactNode;
  prefix?: string;
}

const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, error, hint, icon, prefix, id, ...props }, ref) => {
    const genId = useId();
    const inputId = id || genId;
    return (
      <div className="input-group">
        {label && <label className="input-label" htmlFor={inputId}>{label}</label>}
        <div className="relative">
          {icon && <span className="absolute right-3 top-1/2 -translate-y-1/2" style={{ color: "var(--text-muted)" }}>{icon}</span>}
          {prefix && <span className="absolute right-3 top-1/2 -translate-y-1/2 text-sm" style={{ color: "var(--text-muted)" }}>{prefix}</span>}
          <input
            ref={ref}
            id={inputId}
            className={cn(
              "w-full rounded-xl px-4 py-2.5 text-sm transition-all duration-200",
              "focus:outline-none focus:ring-2",
              icon && "pr-10",
              prefix && "pr-16",
              error ? "border-red-500" : "",
              className
            )}
            style={{
              background: "color-mix(in srgb, var(--surface-card) 70%, var(--surface-bg))",
              border: "1.5px solid var(--border)",
              color: "var(--text-primary)",
              "--tw-ring-color": error ? "color-mix(in srgb, var(--danger) 30%, transparent)" : "color-mix(in srgb, var(--brand-500) 30%, transparent)",
            } as React.CSSProperties}
            aria-invalid={error ? "true" : undefined}
            aria-describedby={error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined}
            {...props}
          />
        </div>
        {error && <p id={`${inputId}-error`} className="text-xs" style={{ color: "var(--danger)" }} role="alert">{error}</p>}
        {hint && !error && <p id={`${inputId}-hint`} className="text-xs" style={{ color: "var(--text-muted)" }}>{hint}</p>}
      </div>
    );
  }
);

Input.displayName = "Input";

interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
}

const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, label, error, id, ...props }, ref) => {
    const genId = useId();
    const textareaId = id || genId;
    return (
      <div className="input-group">
        {label && <label className="input-label" htmlFor={textareaId}>{label}</label>}
        <textarea
          ref={ref}
          id={textareaId}
          className={cn(
            "w-full rounded-xl px-4 py-2.5 text-sm transition-all duration-200 resize-y min-h-[80px]",
            "focus:outline-none focus:ring-2",
            error ? "border-red-500" : "",
            className
          )}
          style={{
            background: "color-mix(in srgb, var(--surface-card) 70%, var(--surface-bg))",
            border: "1.5px solid var(--border)",
            color: "var(--text-primary)",
          }}
          aria-invalid={error ? "true" : undefined}
          aria-describedby={error ? `${textareaId}-error` : undefined}
          {...props}
        />
        {error && <p id={`${textareaId}-error`} className="text-xs" style={{ color: "var(--danger)" }} role="alert">{error}</p>}
      </div>
    );
  }
);

Textarea.displayName = "Textarea";

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  options: { value: string | number; label: string }[];
  placeholder?: string;
}

const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, label, error, options, placeholder, id, ...props }, ref) => {
    const genId = useId();
    const selectId = id || genId;
    return (
      <div className="input-group">
        {label && <label className="input-label" htmlFor={selectId}>{label}</label>}
        <select
          ref={ref}
          id={selectId}
          className={cn(
            "w-full rounded-xl px-4 py-2.5 text-sm transition-all duration-200 appearance-none",
            "focus:outline-none focus:ring-2",
            error ? "border-red-500" : "",
            className
          )}
          style={{
            background: "color-mix(in srgb, var(--surface-card) 70%, var(--surface-bg))",
            border: "1.5px solid var(--border)",
            color: "var(--text-primary)",
          }}
          aria-invalid={error ? "true" : undefined}
          aria-describedby={error ? `${selectId}-error` : undefined}
          {...props}
        >
          {placeholder && <option value="" style={{ color: "var(--text-muted)" }}>{placeholder}</option>}
          {options.map((opt) => (
            <option key={opt.value} value={opt.value} style={{ background: "var(--surface-card)", color: "var(--text-primary)" }}>{opt.label}</option>
          ))}
        </select>
        {error && <p id={`${selectId}-error`} className="text-xs" style={{ color: "var(--danger)" }} role="alert">{error}</p>}
      </div>
    );
  }
);

Select.displayName = "Select";

export { Input, Textarea, Select };
export default Input;

