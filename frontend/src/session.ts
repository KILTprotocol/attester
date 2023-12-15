import { DidUri } from '@kiltprotocol/sdk-js';
import { useState, useEffect } from 'react';
import { getAxiosClient } from './dataProvider';

interface EncryptedMessage {
  receiverKeyUri: string;
  senderKeyUri: string;
  ciphertext: string;
  nonce: string;
  receivedAt?: number;
}

interface PubSubSession {
  listen: (callback: (message: EncryptedMessage) => Promise<void>) => Promise<void>;
  close: () => Promise<void>;
  send: (message: EncryptedMessage) => Promise<void>;
  encryptionKeyUri: string;
  encryptedChallenge: string;
  nonce: string;
}

export interface InjectedWindowProvider {
  startSession: (dAppName: string, dAppEncryptionKeyUri: string, challenge: string) => Promise<PubSubSession>;
  name: string;
  version: string;
  specVersion: '3.0';
  signWithDid: (data: string, didKeyUri: DidUri) => Promise<{ didKeyUri: string; signature: string }>;
  getDidList: () => Promise<Array<{ did: DidUri }>>;
}

export const apiWindow = window as unknown as {
  kilt: Record<string, InjectedWindowProvider>;
};


export function useCompatibleExtensions() {
  const [extensions, setExtensions] = useState(getCompatibleExtensions());
  useEffect(() => {
    function handler() {
      setExtensions(getCompatibleExtensions());
    }
    window.dispatchEvent(new CustomEvent('kilt-dapp#initialized'));
    window.addEventListener('kilt-extension#initialized', handler);
    return () => window.removeEventListener('kilt-extension#initialized', handler);
  }, []);

  return { extensions };
}


export function getCompatibleExtensions(): Array<string> {
  return Object.entries(apiWindow.kilt)
    .filter(([, provider]) => provider.specVersion.startsWith('3.'))
    .map(([name]) => name);
}

export async function getSession(provider: InjectedWindowProvider, attestationId: string): Promise<PubSubSession> {
  if (!provider) {
    throw new Error('No provider');
  }

  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
  const url = apiURL + "/challenge";

  let client = await getAxiosClient();

  let get_challenge_response = await client.get(url);

  if (get_challenge_response.status !== 200) {
    throw new Error('No valid challenge received');
  }

  const challenge = get_challenge_response.data;
  const session = await provider.startSession(challenge.dAppName, challenge.dAppEncryptionKeyUri, challenge.challenge);

  const response = {
    ...session,
    attestationId
  }

  // post challenge and receive encrypted Message.
  let post_session_response = await client.post(url, response);

  if (post_session_response.status !== 200) {
    throw new Error('No valid Session.');
  }

  console.log(post_session_response.data);

  session.send(post_session_response.data);

  return session;
}
