import React, { useEffect, useRef, useState } from 'react'
import { Editor } from '@monaco-editor/react'
import { collaborationService, InitializeSessionPayload, OperationPayload, CursorUpdatePayload } from '../../services/collaboration'
import { invoke } from '@tauri-apps/api/tauri'

interface RealtimeEditorProps {
  documentId: string
  userId: string
  userName: string
  colorHex: string
  permissions: string[]
  serverUrl: string
  sessionToken: string
}

interface CursorPosition {
  clientId: string
  line: number
  col: number
  color: string
  name: string
}

const RealtimeEditor: React.FC<RealtimeEditorProps> = ({
  documentId,
  userId,
  userName,
  colorHex,
  permissions,
  serverUrl,
  sessionToken
}) => {
  const editorRef = useRef<any>(null)
  const [content, setContent] = useState('')
  const [connected, setConnected] = useState(false)
  const [cursors, setCursors] = useState<CursorPosition[]>([])
  const [connecting, setConnecting] = useState(false)
  const [error, setError] = useState<string>('')

  // Connect to collaboration session
  const connect = async () => {
    if (connected) return

    try {
      setConnecting(true)
      setError('')

      await collaborationService.connect({
        serverUrl,
        sessionToken,
        reconnectOnError: true
      })

      setConnected(true)
      console.log('Connected to collaboration server')
    } catch (err: any) {
      console.error('Failed to connect:', err)
      setError('Failed to connect to collaboration server')
    } finally {
      setConnecting(false)
    }
  }

  // Start collaboration session
  const startSession = async () => {
    if (!connected) return

    try {
      const payload: InitializeSessionPayload = {
        document_id: documentId,
        user_id: userId,
        user_name: userName,
        color_hex: colorHex,
        permissions
      }

      await collaborationService.startSession(payload)
      console.log('Collaboration session started')
    } catch (err: any) {
      console.error('Failed to start session:', err)
      setError('Failed to start collaboration session')
    }
  }

  // Send an operation
  const sendOperation = async (operationType: 'insert' | 'delete', position: number, text?: string) => {
    try {
      const payload: OperationPayload = {
        document_id: documentId,
        operation_type: operationType,
        position,
        text,
        client_id: userId
      }

      await collaborationService.sendOperation(payload)
    } catch (err: any) {
      console.error('Failed to send operation:', err)
    }
  }

  // Send cursor update
  const sendCursorUpdate = async (line: number, col: number) => {
    try {
      const payload: CursorUpdatePayload = {
        document_id: documentId,
        client_id: userId,
        line,
        col
      }

      await collaborationService.sendCursorUpdate(payload)
    } catch (err: any) {
      console.error('Failed to send cursor update:', err)
    }
  }

  // End session
  const endSession = async () => {
    try {
      await collaborationService.endSession({
        client_id: userId
      })

      setConnected(false)
      console.log('Collaboration session ended')
    } catch (err: any) {
      console.error('Failed to end session:', err)
    }
  }

  // Get participants (for cursor updates)
  const getCursorPositions = async () => {
    try {
      const response = await collaborationService.getParticipants()
      if (response.success) {
        setCursors(response.participants || [])
      }
    } catch (err: any) {
      console.error('Failed to get participants:', err)
    }
  }

  // Handle editor mount
  const handleEditorDidMount = (editor: any) => {
    editorRef.current = editor

    // Listen for content changes
    editor.onDidChangeModelContent((event: any) => {
      const changes = event.changes[0]
      if (changes) {
        const range = changes.range
        const text = changes.text
        const startOffset = editor.getModel().getOffsetAt(range.getStartPosition())

        if (changes.rangeLength === 0 && text.length > 0) {
          // Insert operation
          sendOperation('insert', startOffset, text)
        } else if (changes.rangeLength > 0 && text.length === 0) {
          // Delete operation
          sendOperation('delete', startOffset, undefined)
        }
      }
    })

    // Listen for cursor position changes
    editor.onDidChangeCursorPosition((event: any) => {
      const position = event.position
      sendCursorUpdate(position.lineNumber - 1, position.column - 1) // Convert to 0-based
    })
  }

  // Initialize editor
  useEffect(() => {
    return () => {
      if (connected) {
        endSession()
      }
    }
  }, [connected])

  // Connect when component mounts
  useEffect(() => {
    connect().then(() => {
      if (connected) {
        startSession()
        // Get current participants
        getCursorPositions()
      }
    })
  }, [])

  // Handle editor mount setup
  const handleEditorWillMount = (monaco: any) => {
    // Setup monaco environment if needed
    // The language is already set via the Editor component props
  }

  return (
    <div className="realtime-editor">
      <div className="editor-toolbar">
        <div className="status">
          {connecting && <span>Connecting...</span>}
          {connected && <span style={{ color: 'green' }}>✓ Connected</span>}
          {!connected && !connecting && <span style={{ color: 'red' }}>✗ Disconnected</span>}
        </div>
        <button
          onClick={connected ? endSession : startSession}
          disabled={connecting}
        >
          {connected ? 'End Session' : 'Start Collaboration'}
        </button>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      <div className="editor-container">
        <Editor
          height="500px"
          language="typescript"
          value={content}
          onChange={(value) => setContent(value || '')}
          onMount={handleEditorDidMount}
          beforeMount={handleEditorWillMount}
          options={{
            minimap: { enabled: true },
            fontSize: 14,
            wordWrap: 'on',
            automaticLayout: true,
          }}
        />

        {/* Render other users' cursors */}
        {cursors.map((cursor) => (
          <div
            key={cursor.clientId}
            className="remote-cursor"
            style={{
              backgroundColor: cursor.color,
              position: 'absolute',
              // These positions would need to be calculated based on Monaco editor coordinates
              // For now, this is a placeholder
              left: `${cursor.col * 8}px`,
              top: `${cursor.line * 16}px`,
              width: '2px',
              height: '16px',
              zIndex: 1000
            }}
            title={`${cursor.name} (${cursor.line}:${cursor.col})`}
          />
        ))}
      </div>
    </div>
  )
}

export default RealtimeEditor