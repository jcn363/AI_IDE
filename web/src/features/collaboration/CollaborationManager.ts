import type {
  UserPresence,
  EditingSession,
  ChangeOperation,
  Conflict,
  CollaborationEvent,
  CollaborationService,
  CollaborationRoom,
} from './types';

export class CollaborationManager implements CollaborationService {
  private currentSession: EditingSession | null = null;
  private userPresence: UserPresence | null = null;
  private changeCallbacks: Set<(operation: ChangeOperation) => void> = new Set();
  private presenceCallbacks: Set<(presence: UserPresence) => void> = new Set();
  private conflictCallbacks: Set<(conflict: Conflict) => void> = new Set();
  private eventCallbacks: Set<(event: CollaborationEvent) => void> = new Set();
  private pendingConficts: Conflict[] = [];
  private operationsHistory: ChangeOperation[] = [];

  // In a real implementation, these would be WebSocket connections
  private mockUsers: Map<string, UserPresence> = new Map();
  private mockSessions: Map<string, EditingSession> = new Map();

  constructor(userData: Omit<UserPresence, 'status' | 'lastSeen' | 'color'>) {
    this.userPresence = {
      ...userData,
      status: 'online',
      lastSeen: Date.now(),
      color: this.generateColor(userData.userId),
    };

    // Initialize with mocked collaborative users for demo
    this.initializeMockUsers();
  }

  async startSession(filePath: string, users?: string[]): Promise<EditingSession> {
    const sessionId = `session_${Date.now()}`;

    const session: EditingSession = {
      sessionId,
      filePath,
      users: users?.map(userId => this.mockUsers.get(userId)!).filter(Boolean) || [],
      isActive: true,
      startTime: Date.now(),
      lastActivity: Date.now(),
      permissions: {},
    };

    if (this.userPresence) {
      session.users.push(this.userPresence);
      session.permissions[this.userPresence.userId] = 'write';
    }

    this.currentSession = session;
    this.mockSessions.set(sessionId, session);

    const event: CollaborationEvent = {
      type: 'user_joined',
      sessionId,
      userId: this.userPresence?.userId || '',
      timestamp: Date.now(),
      data: { filePath },
    };

    this.broadcastEvent(event);
    return session;
  }

  async joinSession(sessionId: string): Promise<void> {
    const session = this.mockSessions.get(sessionId);
    if (!session || !this.userPresence) return;

    session.users.push(this.userPresence);
    this.currentSession = session;

    const event: CollaborationEvent = {
      type: 'user_joined',
      sessionId,
      userId: this.userPresence.userId,
      timestamp: Date.now(),
      data: { filePath: session.filePath },
    };

    this.broadcastEvent(event);
  }

  async leaveSession(): Promise<void> {
    if (!this.currentSession || !this.userPresence) return;

    const session = this.currentSession;
    session.users = session.users.filter(user => user.userId !== this.userPresence!.userId);

    const event: CollaborationEvent = {
      type: 'user_left',
      sessionId: session.sessionId,
      userId: this.userPresence.userId,
      timestamp: Date.now(),
      data: {},
    };

    this.broadcastEvent(event);
    this.currentSession = null;
  }

  async endSession(): Promise<void> {
    if (!this.currentSession) return;

    this.currentSession.isActive = false;
    await this.leaveSession();
  }

  updatePresence(update: Partial<UserPresence>): void {
    if (!this.userPresence) return;

    this.userPresence = { ...this.userPresence, ...update };

    // Broadcast presence update
    this.presenceCallbacks.forEach(callback => callback(this.userPresence!));
  }

  getUsersInFile(filePath: string): UserPresence[] {
    return Array.from(this.mockUsers.values()).filter(
      user => user.currentFile === filePath && user.status === 'online'
    );
  }

  async inviteUser(userId: string, filePath: string): Promise<void> {
    // In real implementation, this would send an invitation
    console.log(`Inviting user ${userId} to collaborate on ${filePath}`);
  }

  sendChange(operation: ChangeOperation): void {
    this.operationsHistory.push(operation);

    // Detect conflicts with simulated remote operations
    const remoteOps = this.simulateRemoteOperation(operation);
    if (remoteOps.length > 0) {
      remoteOps.forEach(remoteOp => {
        const conflict = this.detectConflict(operation, remoteOp);
        if (conflict) {
          this.pendingConficts.push(conflict);
          this.conflictCallbacks.forEach(callback => callback(conflict));
        }
      });
    }

    // Notify listeners
    this.changeCallbacks.forEach(callback => callback(operation));
  }

  broadcastCursor(position: any): void {
    if (!this.userPresence) return;

    const update: Partial<UserPresence> = {
      currentLine: position.lineNumber,
      currentColumn: position.column,
      cursorPosition: {
        startLine: position.lineNumber,
        startColumn: position.column,
        endLine: position.lineNumber,
        endColumn: position.column,
      },
      lastSeen: Date.now(),
    };

    this.updatePresence(update);
  }

  broadcastSelection(selection: any): void {
    if (!this.userPresence) return;

    const update: Partial<UserPresence> = {
      selection: {
        startLine: selection.startLineNumber,
        startColumn: selection.startColumn,
        endLine: selection.endLineNumber,
        endColumn: selection.endColumn,
      },
      lastSeen: Date.now(),
    };

    this.updatePresence(update);
  }

  detectConflict(localChange: ChangeOperation, remoteChange: ChangeOperation): Conflict | null {
    // Simple conflict detection based on overlapping line ranges
    const localRange = localChange.position;
    const remoteRange = remoteChange.position;

    const overlaps = !(
      localRange.endLine < remoteRange.startLine ||
      localRange.startLine > remoteRange.endLine ||
      (localRange.endLine === remoteRange.startLine && localRange.endColumn < remoteRange.startColumn) ||
      (localRange.startLine === remoteRange.endLine && localRange.startColumn > remoteRange.endColumn)
    );

    if (overlaps && localChange.type !== remoteChange.type) {
      return {
        id: `conflict_${Date.now()}`,
        sessionId: this.currentSession?.sessionId || '',
        filePath: localChange.filePath,
        userId: localChange.userId,
        otherUserId: remoteChange.userId,
        timestamp: Date.now(),
        type: remoteChange.type === 'delete' ? 'delete' : 'merge',
        localChange,
        remoteChange,
        resolved: false,
      };
    }

    return null;
  }

  resolveConflict(conflictId: string, resolution: 'local' | 'remote' | 'merge', mergedContent?: string): void {
    const conflict = this.pendingConficts.find(c => c.id === conflictId);
    if (!conflict) return;

    conflict.resolved = true;
    conflict.resolution = resolution;
    if (mergedContent) {
      conflict.mergedContent = mergedContent;
    }

    // Remove from pending conflicts
    this.pendingConficts = this.pendingConficts.filter(c => c.id !== conflictId);

    console.log(`Resolved conflict ${conflictId} with resolution: ${resolution}`);
  }

  getPendingConflicts(): Conflict[] {
    return this.pendingConficts;
  }

  async pullChanges(filePath: string): Promise<ChangeOperation[]> {
    // Simulate pulling changes from server
    return this.operationsHistory.filter(op => op.filePath === filePath);
  }

  async pushChanges(filePath: string): Promise<void> {
    // Simulate pushing changes to server
    console.log(`Pushed changes for ${filePath}`);
  }

  async syncFile(filePath: string): Promise<string> {
    // Simulate file synchronization
    const changes = this.operationsHistory.filter(op => op.filePath === filePath);

    // Apply changes in order to get final content
    let content = '';
    changes.forEach(change => {
      if (change.type === 'insert' && change.content) {
        content = this.applyOperation(content, change);
      }
    });

    return content;
  }

  async createRoom(roomData: Omit<CollaborationRoom, 'id' | 'createdAt'>): Promise<CollaborationRoom> {
    const room: CollaborationRoom = {
      ...roomData,
      id: `room_${Date.now()}`,
      createdAt: Date.now(),
    };

    console.log(`Created collaboration room: ${room.name}`);
    return room;
  }

  async joinRoom(roomId: string): Promise<void> {
    console.log(`Joined room: ${roomId}`);
  }

  async leaveRoom(roomId: string): Promise<void> {
    console.log(`Left room: ${roomId}`);
  }

  async getRoomFiles(roomId: string): Promise<string[]> {
    // Mock room files
    return [`/project/room-${roomId}/main.rs`, '/project/room-${roomId}/lib.rs'];
  }

  onChange(callback: (operation: ChangeOperation) => void): () => void {
    this.changeCallbacks.add(callback);
    return () => this.changeCallbacks.delete(callback);
  }

  onPresenceUpdate(callback: (presence: UserPresence) => void): () => void {
    this.presenceCallbacks.add(callback);
    return () => this.presenceCallbacks.delete(callback);
  }

  onConflict(callback: (conflict: Conflict) => void): void {
    this.conflictCallbacks.add(callback);
  }

  onSessionEvent(callback: (event: CollaborationEvent) => void): () => void {
    this.eventCallbacks.add(callback);
    return () => this.eventCallbacks.delete(callback);
  }

  // Private methods
  private generateColor(seed: string): string {
    const colors = ['#e91e63', '#9c27b0', '#673ab7', '#3f51b5', '#2196f3', '#00bcd4', '#009688', '#4caf50', '#8bc34a', '#cddc39'];
    let hash = 0;
    for (let i = 0; i < seed.length; i++) {
      hash = seed.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
  }

  private initializeMockUsers(): void {
    const mockUsers = [
      {
        userId: 'user_alice',
        name: 'Alice Johnson',
        email: 'alice@example.com',
        currentFile: '/project/src/main.rs',
        currentLine: 15,
        status: 'online' as const,
        lastSeen: Date.now(),
      },
      {
        userId: 'user_bob',
        name: 'Bob Smith',
        email: 'bob@example.com',
        currentFile: '/project/src/lib.rs',
        currentLine: 22,
        status: 'online' as const,
        lastSeen: Date.now(),
      },
    ];

    mockUsers.forEach(user => {
      const presence: UserPresence = {
        ...user,
        color: this.generateColor(user.userId),
        avatar: `https://ui-avatars.com/api/?name=${user.name}&background=${user.userId.slice(-6)}&color=fff`,
      };
      this.mockUsers.set(user.userId, presence);
    });
  }

  private simulateRemoteOperation(localOperation: ChangeOperation): ChangeOperation[] {
    // Randomly simulate remote operations for demo purposes
    if (Math.random() < 0.1) { // 10% chance
      return [{
        id: `remote_${Date.now()}`,
        userId: 'user_simulated',
        filePath: localOperation.filePath,
        timestamp: Date.now() + 100,
        type: 'insert',
        position: {
          startLine: localOperation.position.startLine,
          startColumn: localOperation.position.startColumn + 5,
          endLine: localOperation.position.endLine,
          endColumn: localOperation.position.endColumn + 5,
        },
        content: '// Simulated remote change\n',
      }];
    }
    return [];
  }

  private broadcastEvent(event: CollaborationEvent): void {
    this.eventCallbacks.forEach(callback => callback(event));
  }

  private applyOperation(currentContent: string, operation: ChangeOperation): string {
    if (operation.type === 'insert' && operation.content) {
      // Simple content insertion for demo
      return currentContent + operation.content;
    }
    return currentContent;
  }
}