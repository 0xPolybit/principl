export function PrinciMark({ small = false }: { small?: boolean }) {
  return (
    <svg
      aria-hidden="true"
      className={small ? "princi-mark princi-mark-small" : "princi-mark"}
      viewBox="0 0 40 40"
      fill="none"
    >
      <path
        d="M8 7.5h13.4c8 0 13.1 4.6 13.1 11.6s-5.1 11.7-13.1 11.7h-5.1v5.7H8V7.5Zm8.3 7.2v8.8h4.9c3.3 0 5-1.5 5-4.4s-1.7-4.4-5-4.4h-4.9Z"
        fill="currentColor"
      />
      <circle cx="31.5" cy="8.5" r="3.5" fill="var(--accent)" />
    </svg>
  );
}
