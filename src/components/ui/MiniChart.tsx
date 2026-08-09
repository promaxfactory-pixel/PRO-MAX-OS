import {
  LineChart, Line, AreaChart, Area, BarChart, Bar,
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
} from "recharts";

const COLORS = [
  "var(--mode-accent)",
  "var(--brand-gold)",
  "var(--brand-primary)",
  "#10b981",
  "#ef4444",
  "#f59e0b",
  "#8b5cf6",
  "#06b6d4",
];

const AXIS_TICK = { fill: "var(--text-muted)", fontSize: 10 };

export interface MiniChartProps {
  data: Array<Record<string, any>>;
  xKey: string;
  yKeys: string[];
  type?: "line" | "area" | "bar";
  height?: number;
  showGrid?: boolean;
  showTooltip?: boolean;
}

function ChartTooltip({ active, payload, label }: any) {
  if (!active || !payload || payload.length === 0) return null;
  return (
    <div
      className="rounded-lg p-3 shadow-xl"
      style={{ background: "var(--surface-card)", borderColor: "var(--border)", borderWidth: 1 }}
    >
      <p className="text-xs mb-1" style={{ color: "var(--text-muted)" }}>{label}</p>
      {payload.map((entry: any, i: number) => (
        <p key={i} className="text-sm font-medium" style={{ color: entry.color }}>
          {entry.name}: {entry.value?.toLocaleString?.() ?? entry.value}
        </p>
      ))}
    </div>
  );
}

export function MiniChart({
  data,
  xKey,
  yKeys,
  type = "line",
  height = 120,
  showGrid = false,
  showTooltip = true,
}: MiniChartProps) {
  const margin = { top: 0, right: 0, left: 0, bottom: 0 };

  return (
    <div className="w-full" style={{ height }}>
      <ResponsiveContainer width="100%" height={height}>
        {type === "line" ? (
          <LineChart data={data} margin={margin}>
            {showGrid && <CartesianGrid strokeDasharray="3 3" stroke="var(--grid-line)" vertical={false} />}
            <XAxis dataKey={xKey} stroke="var(--text-muted)" fontSize={10} tickLine={false} axisLine={false} tick={AXIS_TICK} interval="preserveStartEnd" />
            <YAxis stroke="var(--text-muted)" fontSize={10} tickLine={false} axisLine={false} tick={AXIS_TICK} tickFormatter={(v) => v >= 1000 ? (v / 1000).toFixed(1) + "K" : v} />
            {showTooltip && <Tooltip content={<ChartTooltip />} />}
            {yKeys.map((key, i) => (
              <Line
                key={key}
                type="monotone"
                dataKey={key}
                stroke={COLORS[i % COLORS.length]}
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
                dot={false}
                activeDot={{ r: 6, strokeWidth: 2 }}
                animationDuration={800}
                animationEasing="ease-out"
                animationBegin={i * 100}
              />
            ))}
          </LineChart>
        ) : type === "area" ? (
          <AreaChart data={data} margin={margin}>
            {showGrid && <CartesianGrid strokeDasharray="3 3" stroke="var(--grid-line)" vertical={false} />}
            <XAxis dataKey={xKey} stroke="var(--text-muted)" fontSize={10} tickLine={false} axisLine={false} tick={AXIS_TICK} interval="preserveStartEnd" />
            <YAxis stroke="var(--text-muted)" fontSize={10} tickLine={false} axisLine={false} tick={AXIS_TICK} />
            {showTooltip && <Tooltip content={<ChartTooltip />} />}
            {yKeys.map((key, i) => (
              <Area
                key={key}
                type="monotone"
                dataKey={key}
                stroke={COLORS[i % COLORS.length]}
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
                fill={COLORS[i % COLORS.length]}
                fillOpacity={0.15}
                dot={false}
                activeDot={{ r: 6, strokeWidth: 2 }}
                animationDuration={800}
                animationEasing="ease-out"
                animationBegin={i * 100}
              />
            ))}
          </AreaChart>
        ) : (
          <BarChart layout="vertical" data={data} margin={margin}>
            {showGrid && <CartesianGrid strokeDasharray="3 3" stroke="var(--grid-line)" horizontal={false} />}
            <XAxis type="number" stroke="var(--text-muted)" fontSize={10} tickLine={false} axisLine={false} tick={AXIS_TICK} />
            <YAxis dataKey={xKey} type="category" stroke="var(--text-muted)" fontSize={10} tickLine={false} axisLine={false} tick={AXIS_TICK} width={80} />
            {showTooltip && <Tooltip content={<ChartTooltip />} />}
            {yKeys.map((key, i) => (
              <Bar
                key={key}
                dataKey={key}
                fill={COLORS[i % COLORS.length]}
                radius={[4, 4, 0, 0]}
                animationDuration={600}
                animationEasing="ease-out"
                animationBegin={i * 100}
              />
            ))}
          </BarChart>
        )}
      </ResponsiveContainer>
    </div>
  );
}
