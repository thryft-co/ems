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
  "flex items-center gap-3 w-full rounded-[10px] px-3 py-2.5 text-[14px] font-medium transition-all duration-200 ease-out focus:outline-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-offset-1",
  {
    variants: {
      active: {
        true: "bg-primary/10 text-primary font-semibold",
        false:
          "text-foreground/60 hover:bg-secondary/60 hover:text-foreground",
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
        "hidden md:flex flex-col h-full bg-card border-r-[0.5px] border-border/50",
        className,
      )}
    >
      {/* Navigation */}
      <nav className="flex-1 px-3 pt-4 overflow-y-auto">
        <div className="space-y-0.5">
          {items.map((item) => (
            <button
              key={item.id}
              onClick={() => onItemClick(item.id)}
              className={cn(
                sidebarButtonVariants({ active: activeItem === item.id }),
              )}
            >
              {item.icon && (
                <item.icon
                  className={cn(
                    "h-[18px] w-[18px] flex-shrink-0",
                    activeItem === item.id
                      ? "text-primary"
                      : "text-foreground/45",
                  )}
                />
              )}
              <span className="truncate">{item.label}</span>
            </button>
          ))}
        </div>
      </nav>

      {/* Footer */}
      <div className="p-4 mt-auto">
        <p className="text-[11px] text-muted-foreground/50 text-center">
          © 2026 Shishir Dey
        </p>
      </div>
    </div>
  );
}
