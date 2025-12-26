import * as React from "react";
import { cn } from "../../lib/utils";

interface SliderProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onChange"> {
  value: number;
  max?: number;
  step?: number;
  onChange?: (value: number) => void;
}

const Slider = React.forwardRef<HTMLDivElement, SliderProps>(
  ({ className, value, max = 100, step = 1, onChange, ...props }, ref) => {
    const trackRef = React.useRef<HTMLDivElement>(null);

    const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
      if (!trackRef.current || !onChange) return;
      const rect = trackRef.current.getBoundingClientRect();
      const percent = Math.min(Math.max((e.clientX - rect.left) / rect.width, 0), 1);
      onChange(percent * max);
    };

    return (
      <div
        ref={ref}
        className={cn("relative flex w-full touch-none select-none items-center", className)}
        {...props}
      >
        <div
          ref={trackRef}
          className="relative h-2 w-full grow cursor-pointer overflow-hidden rounded-full bg-secondary"
          onClick={handleClick}
        >
          <div
            className="absolute h-full bg-primary transition-all"
            style={{ width: `${(value / max) * 100}%` }}
          />
        </div>
      </div>
    );
  }
);
Slider.displayName = "Slider";

export { Slider };
