import { getAxiosClient } from './dataProvider'
import { InjectedWindowProvider } from '@kiltprotocol/kilt-extension-api'

export async function fetchCredential(extension: InjectedWindowProvider, sessionId: string, attestationId: string) {
  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL

  const client = await getAxiosClient()

  const credentialUrl = `${apiURL}/credential`;

  const getTermsResponse = await client.post(`${credentialUrl}/terms/${sessionId}/${attestationId}`, sessionId);


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

  client.post(`${credentialUrl}/${sessionId}/${attestationId}`, getCredentialRequestFromExtension)
}
