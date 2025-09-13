export interface UserPresence {
  userId: string;
  name: string;
  email: string;
  avatar?: string;
  status: 'online' | 'away' | 'offline';
  lastSeen: number;
  currentFile?: string;
  currentLine?: number;
  currentColumn?: number;
  cursorPosition?: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  selection?: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  color: string; // For distinguishing users
}

export interface EditingSession {
  sessionId: string;
  filePath: string;
  users: UserPresence[];
  isActive: boolean;
  startTime: number;
  lastActivity: number;
  permissions: {
    [userId: string]: 'read' | 'write' | 'admin';
  };
}

export interface ChangeOperation {
  id: string;
  userId: string;
  filePath: string;
  timestamp: number;
  type: 'insert' | 'delete' | 'undo' | 'redo' | 'selection' | 'cursor';
  position: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  content?: string; // For insert operations
  previousContent?: string; // For delete operations
}

export interface Conflict {
  id: string;
  sessionId: string;
  filePath: string;
  userId: string;
  otherUserId: string;
  timestamp: number;
  type: 'merge' | 'overwrite' | 'delete';
  localChange: ChangeOperation;
  remoteChange: ChangeOperation;
  resolved?: boolean;
  resolution?: 'local' | 'remote' | 'merge';
  mergedContent?: string;
}

export interface CollaborationEvent {
  type:
    | 'user_joined'
    | 'user_left'
    | 'user_moved'
    | 'file_opened'
    | 'file_closed'
    | 'change'
    | 'conflict';
  sessionId: string;
  userId: string;
  timestamp: number;
  data: any;
}

export interface CollaborationRoom {
  id: string;
  name: string;
  description?: string;
  owner: string;
  members: string[];
  files: string[];
  isPublic: boolean;
  createdAt: number;
  settings: {
    allowReadByDefault: boolean;
    requireApprovalForWrites: boolean;
    enableConflictResolution: boolean;
    maxMembers?: number;
  };
}

export interface CollaborationState {
  currentSession: EditingSession | null;
  userPresence: UserPresence | null;
  activeRooms: CollaborationRoom[];
  conflicts: Conflict[];
  isConnected: boolean;
  pendingChanges: ChangeOperation[];
  unsavedChanges: Map<string, ChangeOperation[]>;
}

export interface CollaborationService {
  // Session management
  startSession(filePath: string, users?: string[]): Promise<EditingSession>;
  joinSession(sessionId: string): Promise<void>;
  leaveSession(): Promise<void>;
  endSession(): Promise<void>;

  // User presence
  updatePresence(update: Partial<UserPresence>): void;
  getUsersInFile(filePath: string): UserPresence[];
  inviteUser(userId: string, filePath: string): Promise<void>;

  // Real-time operations
  sendChange(operation: ChangeOperation): void;
  broadcastCursor(position: any): void;
  broadcastSelection(selection: any): void;

  // Conflict resolution
  detectConflict(localChange: ChangeOperation, remoteChange: ChangeOperation): Conflict | null;
  resolveConflict(
    conflictId: string,
    resolution: 'local' | 'remote' | 'merge',
    mergedContent?: string
  ): void;
  getPendingConflicts(): Conflict[];

  // File synchronization
  pullChanges(filePath: string): Promise<ChangeOperation[]>;
  pushChanges(filePath: string): Promise<void>;
  syncFile(filePath: string): Promise<string>;

  // Room management
  createRoom(room: Omit<CollaborationRoom, 'id' | 'createdAt'>): Promise<CollaborationRoom>;
  joinRoom(roomId: string): Promise<void>;
  leaveRoom(roomId: string): Promise<void>;
  getRoomFiles(roomId: string): Promise<string[]>;

  // Event handling
  onChange(callback: (operation: ChangeOperation) => void): () => void;
  onPresenceUpdate(callback: (presence: UserPresence) => void): () => void;
  onConflict(callback: (conflict: Conflict) => void): void;
  onSessionEvent(callback: (event: CollaborationEvent) => void): () => void;
}

export type PresenceIndicatorProps = {
  user: UserPresence;
  showAvatar?: boolean;
  showName?: boolean;
  size?: 'small' | 'medium' | 'large';
};

export type ConflictResolverProps = {
  conflict: Conflict;
  onResolve: (resolution: 'local' | 'remote' | 'merge') => void;
  onMerge: (mergedContent: string) => void;
};

export type CollaborationPanelProps = {
  currentFile: string;
  users: UserPresence[];
  conflicts: Conflict[];
  onInviteUser: (userId: string) => void;
  onResolveConflict: (conflictId: string, resolution: string) => void;
};
