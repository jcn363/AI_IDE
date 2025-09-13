// CollaborationPanel.tsx - Real-time collaboration interface component
import React, { Component, createRef } from 'react';
import collaborationService from '../services/collaboration';

interface CollaborationPanelProps {
  sessionId?: string;
  isVisible: boolean;
}

interface CollaborationPanelState {
  messages: ChatMessage[];
  participants: Participant[];
  currentMessage: string;
  isConnected: boolean;
  isTyping: boolean;
  currentUser: string;
  coachingMode: boolean;
}

interface ChatMessage {
  id: string;
  sender: string;
  content: string;
  timestamp: number;
  messageType: MessageType;
  coachingEvent?: CoachingEvent;
}

interface Participant {
  id: string;
  name: string;
  avatar?: string;
  isOnline: boolean;
  role: ParticipantRole;
  cursor?: { row: number; col: number };
}

enum MessageType {
  CHAT = 'chat',
  SYSTEM = 'system',
  COACHING = 'coaching',
  EDITING = 'editing',
}

enum ParticipantRole {
  HOST = 'host',
  COLLABORATOR = 'collaborator',
  AI_AGENT = 'ai_agent',
  OBSERVER = 'observer',
}

interface CoachingEvent {
  type: string;
  content: string;
  confidence?: number;
  context?: string;
}

class CollaborationPanel extends Component<CollaborationPanelProps, CollaborationPanelState> {
  private chatContainerRef = createRef<HTMLDivElement>();
  private typingTimeout: NodeJS.Timeout | null = null;
  private eventListeners: (() => void)[] = [];

  constructor(props: CollaborationPanelProps) {
    super(props);

    this.state = {
      messages: [],
      participants: [],
      currentMessage: '',
      isConnected: false,
      isTyping: false,
      currentUser: this.getCurrentUser(),
      coachingMode: true,
    };
  }

  componentDidMount() {
    this.subscribeToCollaborationEvents();
    this.loadInitialState();
  }

  componentWillUnmount() {
    this.cleanupEventListeners();
  }

  componentDidUpdate(prevProps: CollaborationPanelProps, prevState: CollaborationPanelState) {
    // Auto-scroll to bottom on new messages
    if (prevState.messages.length < this.state.messages.length) {
      this.scrollToBottom();
    }

    // Handle visibility changes
    if (prevProps.isVisible !== this.props.isVisible) {
      this.handleVisibilityChange();
    }
  }

  private async loadInitialState() {
    // Load current chat messages and participants
    const messages = await this.props.service.getRecentMessages(this.props.sessionId || '');
    const participants = await this.props.service.getSessionParticipants(
      this.props.sessionId || ''
    );

    this.setState({
      messages: messages || [],
      participants: participants || [],
      isConnected: Boolean(this.props.sessionId),
    });
  }

  private subscribeToCollaborationEvents() {
    // Subscribe to real-time events
    const unsubscribeMessage = this.props.service.onMessage((message: ChatMessage) => {
      this.setState((prevState) => ({
        messages: [...prevState.messages, message],
      }));
    });

    const unsubscribeParticipantUpdate = this.props.service.onParticipantUpdate(
      (participants: Participant[]) => {
        this.setState({ participants });
      }
    );

    const unsubscribeCoachingEvent = this.props.service.onCoachingEvent((event: CoachingEvent) => {
      const coachingMessage: ChatMessage = {
        id: Date.now().toString(),
        sender: 'AI Coach',
        content: '',
        timestamp: Date.now(),
        messageType: MessageType.COACHING,
        coachingEvent: event,
      };
      this.setState((prevState) => ({
        messages: [...prevState.messages, coachingMessage],
      }));
    });

    this.eventListeners = [
      unsubscribeMessage,
      unsubscribeParticipantUpdate,
      unsubscribeCoachingEvent,
    ];
  }

  private cleanupEventListeners() {
    this.eventListeners.forEach((unsubscribe) => unsubscribe());
    if (this.typingTimeout) {
      clearTimeout(this.typingTimeout);
    }
  }

  private handleVisibilityChange() {
    if (this.props.isVisible) {
      this.scrollToBottom();
      this.focusMessageInput();
    }
  }

  private getCurrentUser(): string {
    // Get current user from application context
    // Placeholder implementation
    return 'User'; // This would come from app state
  }

  private handleMessageChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = event.target.value;
    this.setState({ currentMessage: value });

    // Handle typing indicator
    if (!this.state.isTyping) {
      this.props.service.sendTypingIndicator(true);
      this.setState({ isTyping: true });
    }

    // Reset typing timeout
    if (this.typingTimeout) {
      clearTimeout(this.typingTimeout);
    }

    this.typingTimeout = setTimeout(() => {
      this.props.service.sendTypingIndicator(false);
      this.setState({ isTyping: false });
    }, 1000);
  };

  private handleSendMessage = async () => {
    if (!this.state.currentMessage.trim()) return;

    const message: ChatMessage = {
      id: Date.now().toString(),
      sender: this.state.currentUser,
      content: this.state.currentMessage.trim(),
      timestamp: Date.now(),
      messageType: MessageType.CHAT,
    };

    try {
      await this.props.service.sendMessage(message);
      this.setState({ currentMessage: '' });
      // Stop typing indicator
      this.props.service.sendTypingIndicator(false);
      this.setState({ isTyping: false });
    } catch (error) {
      console.error('Failed to send message:', error);
      // Could add error handling UI here
    }
  };

  private handleKeyPress = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      this.handleSendMessage();
    }
  };

  private scrollToBottom() {
    if (this.chatContainerRef.current) {
      this.chatContainerRef.current.scrollTop = this.chatContainerRef.current.scrollHeight;
    }
  }

  private focusMessageInput() {
    // Focus message input when panel becomes visible
    const inputElement = document.getElementById(
      'collaboration-message-input'
    ) as HTMLTextAreaElement;
    if (inputElement) {
      inputElement.focus();
    }
  }

  private toggleCoachingMode = () => {
    this.setState((prevState) => ({ coachingMode: !prevState.coachingMode }));

    // Notify backend about coaching mode change
    this.props.service.updateCoachingMode(this.state.coachingMode);
  };

  private renderMessage(message: ChatMessage): React.ReactNode {
    const isOwnMessage = message.sender === this.state.currentUser;

    if (message.messageType === MessageType.COACHING && message.coachingEvent) {
      return (
        <div key={message.id} className="coaching-message">
          <div className="coaching-header">
            <span className="ai-icon">🤖</span>
            <span className="coaching-label">AI Coaching</span>
            <span className="coaching-confidence">
              {message.coachingEvent.confidence &&
                `${Math.round(message.coachingEvent.confidence * 100)}%`}
            </span>
          </div>
          <div className="coaching-content">
            {message.coachingEvent.content}
            {message.coachingEvent.context && (
              <div className="coaching-context">
                <small>Context: {message.coachingEvent.context}</small>
              </div>
            )}
          </div>
        </div>
      );
    }

    return (
      <div key={message.id} className={`message ${isOwnMessage ? 'own-message' : 'other-message'}`}>
        <div className="message-header">
          <span className="sender">{message.sender}</span>
          <span className="timestamp">{new Date(message.timestamp).toLocaleTimeString()}</span>
        </div>
        <div className="message-content">{message.content}</div>
      </div>
    );
  }

  private renderParticipantList(): React.ReactNode {
    return (
      <div className="participants-list">
        <h4>Participants ({this.state.participants.length})</h4>
        {this.state.participants.map((participant) => (
          <div
            key={participant.id}
            className={`participant ${participant.isOnline ? 'online' : 'offline'}`}
          >
            <div className="participant-avatar">
              {participant.avatar ? (
                <img src={participant.avatar} alt={`${participant.name} avatar`} />
              ) : (
                <div className="avatar-placeholder">{participant.name.charAt(0).toUpperCase()}</div>
              )}
            </div>
            <div className="participant-info">
              <span className="participant-name">{participant.name}</span>
              <span className={`participant-role ${participant.role}`}>{participant.role}</span>
            </div>
            {participant.isOnline && <div className="online-indicator"></div>}
          </div>
        ))}
      </div>
    );
  }

  render() {
    if (!this.props.isVisible) {
      return null;
    }

    const connectionStatus = this.state.isConnected ? 'Connected' : 'Disconnected';

    return (
      <div className="collaboration-panel">
        {/* Panel Header */}
        <div className="panel-header">
          <h3>Collaborative Session</h3>
          <div className="panel-controls">
            <div
              className={`connection-status ${this.state.isConnected ? 'connected' : 'disconnected'}`}
            >
              {connectionStatus}
            </div>
            <button
              className={`coaching-toggle ${this.state.coachingMode ? 'active' : ''}`}
              onClick={this.toggleCoachingMode}
              title="Toggle AI Coaching Mode"
            >
              AI Coach
            </button>
          </div>
        </div>

        {/* Participants Sidebar */}
        <div className="panel-sidebar">{this.renderParticipantList()}</div>

        {/* Main Chat Area */}
        <div className="chat-container">
          <div className="messages-list" ref={this.chatContainerRef}>
            {this.state.messages.map((message) => this.renderMessage(message))}
          </div>

          {/* Message Input */}
          <div className="message-input-container">
            <textarea
              id="collaboration-message-input"
              value={this.state.currentMessage}
              onChange={this.handleMessageChange}
              onKeyPress={this.handleKeyPress}
              placeholder="Type your message..."
              rows={3}
              disabled={!this.state.isConnected}
            />
            <button
              className="send-button"
              onClick={this.handleSendMessage}
              disabled={!this.state.currentMessage.trim() || !this.state.isConnected}
            >
              Send
            </button>
          </div>
        </div>

        {/* AI Coaching Sidebar */}
        {this.state.coachingMode && (
          <div className="coaching-sidebar">
            <h4>AI Coaching</h4>
            <div className="coaching-status">
              <span>Active Mode:</span>
              <span>Context-Aware</span>
            </div>
            <div className="coaching-options">
              <button className="coach-button">Request Help</button>
              <button className="coach-button">Code Review</button>
              <button className="coach-button">Suggest Improvements</button>
            </div>
          </div>
        )}
      </div>
    );
  }
}

export default CollaborationPanel;
