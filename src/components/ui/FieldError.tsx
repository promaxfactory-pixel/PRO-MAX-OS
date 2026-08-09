import { AlertCircle } from "lucide-react";

export default function FieldError({ message }: { message?: string }) {
  if (!message) return null;
  return (
    <p className="mt-1 text-xs text-red-400 flex items-center gap-1">
      <AlertCircle className="w-3 h-3 shrink-0" />
      {message}
    </p>
  );
}
