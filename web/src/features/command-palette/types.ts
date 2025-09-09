export interface CommandPaletteItem {
  id: string;
  name: string;
  description: string;
  icon: string;
  handler: (editor: any) => Promise<void>;
  disabled?: boolean;
  category?: string;
  keywords?: string[];
  shortcut?: string;
}
