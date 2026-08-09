import { InputHTMLAttributes, TextareaHTMLAttributes, SelectHTMLAttributes, forwardRef } from "react";
import { cn } from "@/lib/utils";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
  icon?: React.ReactNode;
  prefix?: string;
}

const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, error, hint, icon, prefix, ...props }, ref) => {
    return (
      <div className="input-group">
        {label && <label className="input-label">{label}</label>}
        <div className="relative">
          {icon && <span className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] pointer-events-none">{icon}</span>}
          {prefix && <span className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] text-sm pointer-events-none">{prefix}</span>}
          <input
            ref={ref}
            className={cn(
              'input-field',
              icon && 'pr-10',
              prefix && 'pr-16',
              error && 'error',
              className
            )}
            {...props}
          />
        </div>
        {error && <p className="text-xs text-red-400 font-medium" role="alert">{error}</p>}
        {hint && !error && <p className="text-xs text-[var(--text-muted)]">{hint}</p>}
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
  ({ className, label, error, ...props }, ref) => {
    return (
      <div className="input-group">
        {label && <label className="input-label">{label}</label>}
        <textarea
          ref={ref}
          className={cn(
            'input-field resize-y min-h-[80px]',
            error && 'error',
            className
          )}
          {...props}
        />
        {error && <p className="text-xs text-red-400 font-medium" role="alert">{error}</p>}
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
  ({ className, label, error, options, placeholder, ...props }, ref) => {
    return (
      <div className="input-group">
        {label && <label className="input-label">{label}</label>}
        <select
          ref={ref}
          className={cn(
            'select-field',
            error && 'error',
            className
          )}
          {...props}
        >
          {placeholder && <option value="">{placeholder}</option>}
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
        {error && <p className="text-xs text-red-400 font-medium" role="alert">{error}</p>}
      </div>
    );
  }
);

Select.displayName = "Select";

export { Input, Textarea, Select };
export default Input;
