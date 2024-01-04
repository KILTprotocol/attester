import { DidUri, ICType, ICredential } from '@kiltprotocol/sdk-js'
import { UUID } from 'crypto'

export interface AttestationRequest {
  approved: boolean
  revoked: boolean
  claimer: DidUri
  createdAt: string
  credential: ICredential
  ctypeHash: ICType['$id']
  deletedAt?: string
  id: UUID
  updatedAt?: string
  approvedAt?: string
  revokedAt?: string
  txState: string
}
