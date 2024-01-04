import { getAxiosClient } from './dataProvider'
import { InjectedWindowProvider } from '@kiltprotocol/kilt-extension-api'

export async function fetchCredential(extension: InjectedWindowProvider) {
  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL

  const client = await getAxiosClient()

  const credentialUrl = apiURL + '/credential'

  const getTermsResponse = await client.post(
    credentialUrl + '/terms',
    extension
  )

  console.log('terms requests:', getTermsResponse)

  const getCredentialRequestFromExtension = await new Promise(
    (resolve, reject) => {
      try {
        extension.listen(async (credentialRequest: unknown) => {
          resolve(credentialRequest)
        })
        extension.send(getTermsResponse.data)
      } catch (e) {
        reject(e)
      }
    }
  )

  const attestationMessage = await client.post(
    credentialUrl,
    getCredentialRequestFromExtension
  )

  console.log('Final message: ', attestationMessage)
}
