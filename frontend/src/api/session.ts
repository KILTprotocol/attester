import { getAxiosClient } from './dataProvider'

import {
  InjectedWindowProvider,
  PubSubSessionV1,
  PubSubSessionV2,
} from '@kiltprotocol/kilt-extension-api'

export async function getSession(
  provider: InjectedWindowProvider,
  attestationId: string
): Promise<PubSubSessionV1 | PubSubSessionV2> {
  if (!provider) {
    throw new Error('No provider')
  }

  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL
  const challengeUrl = apiURL + '/challenge'

  const client = await getAxiosClient()

  const getChallengeReponse = await client.get(challengeUrl)

  if (getChallengeReponse.status !== 200) {
    throw new Error('No valid challenge received')
  }

  const { dAppName, dAppEncryptionKeyUri, challenge } = getChallengeReponse.data

  const session = await provider.startSession(
    dAppName,
    dAppEncryptionKeyUri,
    challenge
  )

  console.log('Here is the session', session)

  // post challenge and receive encrypted Message.
  const sessionVerification = await client.post(challengeUrl, session)

  if (sessionVerification.status !== 200) {
    throw new Error('No valid Session.')
  }

  return { session, attestationId }
}
