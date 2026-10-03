"use client";

import { useId, useRef, useState } from "react";
import type { KeyboardEvent, ReactNode } from "react";

export type TabItem = {
  id: string;
  label: string;
  content: ReactNode;
};

export function Tabs({
  label,
  items,
}: {
  label: string;
  items: readonly TabItem[];
}) {
  const idPrefix = useId();
  const [activeId, setActiveId] = useState(items[0]?.id ?? "");
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const activeIndex = Math.max(0, items.findIndex((item) => item.id === activeId));
  const activeItem = items[activeIndex];

  function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    let nextIndex: number | undefined;

    if (event.key === "ArrowRight") nextIndex = (activeIndex + 1) % items.length;
    if (event.key === "ArrowLeft") nextIndex = (activeIndex - 1 + items.length) % items.length;
    if (event.key === "Home") nextIndex = 0;
    if (event.key === "End") nextIndex = items.length - 1;

    if (nextIndex === undefined) return;
    event.preventDefault();
    const nextItem = items[nextIndex];
    setActiveId(nextItem.id);
    tabRefs.current[nextIndex]?.focus();
  }

  if (!items.length) return null;

  return (
    <div className="tabs">
      <div aria-label={label} className="tab-list" role="tablist">
        {items.map((item, index) => {
          const selected = item.id === activeItem.id;
          const tabId = `${idPrefix}-tab-${item.id}`;
          const panelId = `${idPrefix}-panel-${item.id}`;

          return (
            <button
              aria-controls={panelId}
              aria-selected={selected}
              className="tab-button"
              id={tabId}
              key={item.id}
              onClick={() => setActiveId(item.id)}
              onKeyDown={handleKeyDown}
              ref={(element) => {
                tabRefs.current[index] = element;
              }}
              role="tab"
              tabIndex={selected ? 0 : -1}
              type="button"
            >
              {item.label}
            </button>
          );
        })}
      </div>
      <div
        aria-labelledby={`${idPrefix}-tab-${activeItem.id}`}
        className="tab-panel"
        id={`${idPrefix}-panel-${activeItem.id}`}
        role="tabpanel"
        tabIndex={0}
      >
        {activeItem.content}
      </div>
    </div>
  );
}
