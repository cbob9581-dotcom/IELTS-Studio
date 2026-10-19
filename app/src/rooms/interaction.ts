export function shouldOpenPaper(e: {
  key: string;
  repeat: boolean;
  focused: boolean;
  editing: boolean;
}): boolean {
  return e.key.toLowerCase() === "e" && !e.repeat && e.focused && !e.editing;
}
