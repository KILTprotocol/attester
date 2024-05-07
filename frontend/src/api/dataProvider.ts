import simpleRestProvider from 'ra-data-simple-rest'
import authProvider from './authProvider'
import { fetchUtils } from 'react-admin'
import axios from 'axios'
import { getBackendUrl } from '../utils/utils'

export async function getAxiosClient() {
  const token = await authProvider.getToken()
  const instance = axios.create({
    headers: {
      Authorization: `Bearer ${token}`,
    },
    withCredentials: true,
  })
  return instance
}

async function httpClient(url: string, options: { [key: string]: any } = {}) {
  const token = await authProvider.getToken()
  if (!options.headers) {
    options.headers = new Headers({ Accept: 'application/json' })
  }
  options.user = {
    authenticated: true,
    token: `Bearer ${token}`,
  }
  return fetchUtils.fetchJson(url, options)
}

const apiUrl = getBackendUrl()
export const dataProvider = simpleRestProvider(apiUrl, httpClient)
