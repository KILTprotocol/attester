import { getBackendUrl } from '../utils/utils'
import { getAxiosClient } from './dataProvider'
import { InjectedWindowProvider } from '@kiltprotocol/kilt-extension-api'

export async function fetchCredential(extension: InjectedWindowProvider, attestationId: string) {
  const apiUrl = getBackendUrl()

  const client = await getAxiosClient()

  const credentialUrl = `${apiUrl}/credential`

  const getTermsResponse = await client.post(`${credentialUrl}/terms/${attestationId}`)

  const getCredentialRequestFromExtension = await new Promise((resolve, reject) => {
    try {
      extension.listen(async (credentialRequest: unknown) => {
        resolve(credentialRequest)
      })
      extension.send(getTermsResponse.data)
    } catch (e) {
      reject(e)
    }
  })

  client.post(`${credentialUrl}/${attestationId}`, getCredentialRequestFromExtension)
}
