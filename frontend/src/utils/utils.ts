import { ICType, CType, connect } from '@kiltprotocol/sdk-js'
import authProvider from '../api/authProvider'

export async function fetchCType(ctypeId: ICType['$id']): Promise<CType.ICTypeDetails> {
  await connect(import.meta.env.VITE_WSS_ENDPOINT)
  return CType.fetchFromChain(ctypeId)
}

export function isUserAdmin() {
  const role = authProvider.getRole()
  return role === 'admin'
}
