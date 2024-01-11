import { DidUri, ICType, ICredential } from '@kiltprotocol/sdk-js'
import { UUID } from 'crypto'

export interface AttestationRequest {
  approved: boolean
  revoked: boolean
  marked_approve: boolean
  claimer: DidUri
  createdAt: string
  credential: ICredential
  ctype_hash: ICType['$id']
  deleted_at?: string
  id: UUID
  approved_at?: string
  revoked_at?: string
  txState: string
}
