import React from "react";
import { cva } from "class-variance-authority";
import { cn } from "@/shared/utils";

interface SidebarItem {
  id: string;
  label: string;
  icon?: React.ComponentType<{
    className?: string;
    style?: React.CSSProperties;
  }>;
}

const sidebarButtonVariants = cva(
  "flex items-center gap-3 w-full rounded-xl px-3 py-2.5 text-sm font-medium transition-all duration-200 ease-out animate-fade-up focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
  {
    variants: {
      active: {
        true: "bg-primary text-primary-foreground shadow-soft border border-foreground/50",
        false:
          "text-muted-foreground hover:bg-accent hover:text-accent-foreground hover:translate-x-0.5",
      },
    },
    defaultVariants: {
      active: false,
    },
  },
);

interface SidebarProps {
  className?: string;
  items: SidebarItem[];
  activeItem: string;
  onItemClick: (id: string) => void;
}

export function Sidebar({
  className,
  items,
  activeItem,
  onItemClick,
}: SidebarProps) {
  return (
    <div
      className={cn(
        "flex flex-col h-full bg-card/80 backdrop-blur-xl border-r border-border/50 shadow-soft",
        className,
      )}
    >
      {/* Navigation */}
      <nav className="flex-1 px-6 pt-5 overflow-y-auto">
        <div className="space-y-1">
          {items.map((item, index) => (
            <button
              key={item.id}
              onClick={() => onItemClick(item.id)}
              style={{ animationDelay: `${index * 30}ms` }}
              className={cn(
                sidebarButtonVariants({ active: activeItem === item.id }),
              )}
            >
              {item.icon && (
                <item.icon
                  className={cn(
                    "h-4 w-4 transition-transform duration-200",
                    activeItem === item.id ? "scale-110" : "text-primary",
                  )}
                />
              )}
              <span className="truncate">{item.label}</span>
              {activeItem === item.id && (
                <span className="ml-auto w-1.5 h-1.5 rounded-full bg-primary-foreground/80 animate-scale-in" />
              )}
            </button>
          ))}
        </div>
      </nav>

      {/* Footer */}
      <div className="p-4 mt-auto border-t border-border/50">
        <p className="text-xs text-muted-foreground/70 text-center tracking-wide">
          © 2026 Shishir Dey
        </p>
      </div>
    </div>
  );
}
