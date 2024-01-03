import { DidUri, ICType, ICredential } from '@kiltprotocol/sdk-js'
import { UUID } from 'crypto'

export interface AttestationRequest {
  approved: boolean
  revoked: boolean
  claimer: DidUri
  created_at: string
  credential: ICredential
  ctype_hash: ICType['$id']
  deleted_at?: string
  id: UUID
  updated_at?: string
  approved_at?: string
  revoked_at?: string
  tx_state: string
}
