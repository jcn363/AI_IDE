// Collaborative editing service using Tauri backend commands

import { invoke } from '@tauri-apps/api/tauri'

// Type definitions for payloads
export interface InitializeSessionPayload {
  document_id: string
  user_id: string
  user_name: string
  color_hex: string
  permissions: string[]
}

export interface OperationPayload {
  document_id: string
  operation_type: 'insert' | 'delete'
  position: number
  text?: string
  client_id: string
}

export interface CursorUpdatePayload {
  document_id: string
  client_id: string
  line: number
  col: number
}

export interface ConnectPayload {
  serverUrl: string
  sessionToken: string
  reconnectOnError: boolean
}

export interface EndSessionPayload {
  client_id: string
}

class CollaborationService {
  // Connect to collaboration server
  async connect(payload: ConnectPayload): Promise<any> {
    return await invoke('collaboration_connect', payload)
  }

  // Start a new collaboration session
  async startSession(payload: InitializeSessionPayload): Promise<any> {
    return await invoke('collaboration_start_session', payload)
  }

  // Send an operation (insert/delete)
  async sendOperation(payload: OperationPayload): Promise<any> {
    return await invoke('collaboration_send_operation', payload)
  }

  // Send cursor position update
  async sendCursorUpdate(payload: CursorUpdatePayload): Promise<any> {
    return await invoke('collaboration_send_cursor_update', payload)
  }

  // End collaboration session
  async endSession(payload: EndSessionPayload): Promise<any> {
    return await invoke('collaboration_end_session', payload)
  }

  // Get participants
  async getParticipants(): Promise<any> {
    return await invoke('collaboration_get_participants', {})
  }

  // Request lock on a section
  async requestLock(sectionStart: number, sectionEnd: number): Promise<any> {
    return await invoke('collaboration_request_lock', {
      section_start: sectionStart,
      section_end: sectionEnd
    })
  }

  // Release lock on a section
  async releaseLock(sectionStart: number, sectionEnd: number): Promise<any> {
    return await invoke('collaboration_release_lock', {
      section_start: sectionStart,
      section_end: sectionEnd
    })
  }
}

export const collaborationService = new CollaborationService()
export default collaborationService