import axios from 'axios'

export async function getEndpoints(): Promise<Array<string>> {
  const origin = window.location.origin
  const backendUrl = `${origin}/api/v1`
  const endpointUrl = `${backendUrl}/endpoints`
  const response = await axios.get<Array<string>>(endpointUrl)

  if (response.status !== 200) {
    throw new Error('Could not fetch endpoints')
  }

  const endpoints = response.data
  endpoints.push(backendUrl)

  return endpoints
}
