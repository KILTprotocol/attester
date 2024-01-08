import { DidUri } from "@kiltprotocol/sdk-js";
import { useState, useEffect } from "react";
import { getAxiosClient } from "./api/dataProvider";

interface EncryptedMessage {
  receiverKeyUri: string;
  senderKeyUri: string;
  ciphertext: string;
  nonce: string;
  receivedAt?: number;
}

interface PubSubSession {
  listen: (
    callback: (message: EncryptedMessage) => Promise<void>
  ) => Promise<void>;
  close: () => Promise<void>;
  send: (message: EncryptedMessage) => Promise<void>;
  encryptionKeyUri: string;
  encryptedChallenge: string;
  nonce: string;
}

export interface InjectedWindowProvider {
  startSession: (
    dAppName: string,
    dAppEncryptionKeyUri: string,
    challenge: string
  ) => Promise<PubSubSession>;
  name: string;
  version: string;
  specVersion: "3.0";
  signWithDid: (
    data: string,
    didKeyUri: DidUri
  ) => Promise<{ didKeyUri: string; signature: string }>;
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
    window.dispatchEvent(new CustomEvent("kilt-dapp#initialized"));
    window.addEventListener("kilt-extension#initialized", handler);
    return () =>
      window.removeEventListener("kilt-extension#initialized", handler);
  }, []);

  return { extensions };
}

export function getCompatibleExtensions(): Array<string> {
  return Object.entries(apiWindow.kilt)
    .filter(([, provider]) => provider.specVersion.startsWith("3."))
    .map(([name]) => name);
}

export async function requestAttestation(
  provider: InjectedWindowProvider,
  attestationId: string
): Promise<void> {
  if (!provider) {
    throw new Error("No provider");
  }

  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
  const challengeUrl = apiURL + "/challenge";


  const client = await getAxiosClient();

  const get_challenge_response = await client.get(challengeUrl);

  if (get_challenge_response.status !== 200) {
    throw new Error("No valid challenge received");
  }

  const challenge = get_challenge_response.data;
  const session = await provider.startSession(
    challenge.dAppName,
    challenge.dAppEncryptionKeyUri,
    challenge.challenge
  );

  // post challenge and receive encrypted Message.
  const post_session_response = await client.post(challengeUrl, session);

  if (post_session_response.status !== 200) {
    throw new Error("No valid Session.");
  }

  const session_reference = post_session_response.data;

  const termsRequestData = {
    challenge: session_reference,
    attestationId,
  };

  const credentialUrl = apiURL + "/credential";

  const get_terms_response = await client.post(
    credentialUrl + "/terms/" + session_reference + "/" + attestationId,
    termsRequestData
  );

  const getCredentialRequestFromExtension = await new Promise(
    async (resolve, reject) => {
      try {
        await session.listen(async (credentialRequest) => {
          resolve(credentialRequest);
        });
        await session.send(get_terms_response.data);
      } catch (e) {
        reject(e);
      }
    }
  );

  let attestation_message = await client.post(
    credentialUrl + "/" + session_reference + "/" + attestationId,
    getCredentialRequestFromExtension
  );

  if (attestation_message.status !== 200) {
    throw new Error("No valid attestation message");
  }

}
