import { List, Datagrid, TextField, DateField, useRecordContext, useNotify, useTheme, useRefresh } from 'react-admin'
import ReactJson from 'react-json-view'
import Fab from '@mui/material/Fab'
import DoneIcon from '@mui/icons-material/Done'
import RemoveIcon from '@mui/icons-material/Remove'
import CircularProgress from '@mui/material/CircularProgress'
import Tooltip from '@mui/material/Tooltip'
import DownloadIcon from '@mui/icons-material/Download'
import BookmarkAddedIcon from '@mui/icons-material/BookmarkAdded'
import { ICType } from '@kiltprotocol/sdk-js'
import { getExtensions } from '@kiltprotocol/kilt-extension-api'

import { AttestationRequest } from '../utils/types'
import { useState } from 'react'
import { getAxiosClient } from '../api/dataProvider'
import { getSession } from '../api/session'
import { getBackendUrl, isUserAdmin } from '../utils/utils'
import { InjectedWindowProvider } from '@kiltprotocol/kilt-extension-api'
import { fetchCredential } from '../api/credential'

export default function AttestationList() {
  const apiUrl = getBackendUrl()

  const ExpandAttestation = () => {
    const record = useRecordContext<AttestationRequest>()
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const [theme, _] = useTheme()
    return <ReactJson theme={theme === 'dark' ? 'colors' : 'bright:inverted'} src={record.credential.claim.contents} />
  }

  const ApproveButton = () => {
    const record = useRecordContext<AttestationRequest>()

    const isApproved = record.marked_approve

    if (isApproved) {
      return <ClaimButton />
    }
    return <MarkApproveButton />
  }

  const ClaimButton = () => {
    const record = useRecordContext<AttestationRequest>()
    const [isLoading, setIsLoading] = useState(false)
    const notify = useNotify()
    const refresh = useRefresh()

    const handleClick = async () => {
      if (isLoading) {
        return
      }

      setIsLoading(true)
      const client = await getAxiosClient()
      await client.put(`${apiUrl}/attestation_request/${record.id}/approve`)
      setTimeout(() => {
        setIsLoading(false)
        refresh()
        notify('Transaction finished')
      }, 60_000)
      refresh()
      notify('Transaction for approval is fired')
    }

    return (
      <Tooltip title="Approve">
        <span>
          <Fab
            color="primary"
            aria-label="add"
            size="small"
            disabled={record.approved || isLoading}
            onClick={handleClick}
            sx={{ marginLeft: '1em', marginRight: '1em' }}
          >
            {isLoading ? <CircularProgress color="error" /> : <DoneIcon />}
          </Fab>
        </span>
      </Tooltip>
    )
  }

  const MarkApproveButton = () => {
    const record = useRecordContext<AttestationRequest>()
    const [isLoading, setIsLoading] = useState(false)
    const notify = useNotify()
    const refresh = useRefresh()

    const handleClick = async () => {
      if (isLoading) {
        return
      }
      setIsLoading(true)
      const client = await getAxiosClient()
      await client.put(`${apiUrl}/attestation_request/${record.id}/mark_approve`)
      refresh()
      notify('Marked as claimable')
      setIsLoading(false)
    }

    return (
      <Tooltip title="Mark claimable">
        <span>
          <Fab
            color="primary"
            aria-label="add"
            size="small"
            disabled={record.approved || isLoading || record.marked_approve}
            onClick={handleClick}
            sx={{ marginLeft: '1em', marginRight: '1em' }}
          >
            {isLoading ? <CircularProgress color="error" /> : <BookmarkAddedIcon />}
          </Fab>
        </span>
      </Tooltip>
    )
  }

  const RevokeButton = () => {
    const record = useRecordContext<AttestationRequest>()
    const [isLoading, setIsLoading] = useState(false)
    const notify = useNotify()
    const refresh = useRefresh()

    const handleClick = async () => {
      if (isLoading) {
        return
      }
      setIsLoading(true)
      const client = await getAxiosClient()

      await client.put(`${apiUrl}/attestation_request/${record.id}/revoke`)

      setTimeout(() => {
        setIsLoading(false)
        refresh()
        notify('Transaction is finished!')
      }, 60_000)
      notify('Transaction for revokation is fired.')
    }

    return (
      <Tooltip title="Revoke">
        <span>
          <Fab
            color="error"
            aria-label="revoke"
            size="small"
            disabled={!record.approved || record.revoked || isLoading}
            onClick={handleClick}
            sx={{ marginLeft: '1em', marginRight: '1em' }}
          >
            {isLoading ? <CircularProgress /> : <RemoveIcon />}
          </Fab>
        </span>
      </Tooltip>
    )
  }

  const DownloadCredential = () => {
    const record = useRecordContext<AttestationRequest>()
    const [isLoading, setIsLoading] = useState(false)
    const notify = useNotify()
    const refresh = useRefresh()

    const extensions = getExtensions()
    const hasExtension = extensions.length > 0

    const handleClick = async () => {
      setIsLoading(true)
      const extensionName = 'Sporran'
      const extension: InjectedWindowProvider = extensions.find((val) => val.name === extensionName)

      try {
        const { session, sessionId } = await getSession(extension)

        await fetchCredential(session, sessionId, record.id)
        refresh()
        notify('Claim created')
        setIsLoading(false)
      } catch (e) {
        console.error(e)
        notify('Could not claim credential.', { type: 'error' })
        setIsLoading(false)
      }
    }

    return (
      <>
        {hasExtension && (
          <Tooltip title="Claim it">
            <Fab
              color="success"
              aria-label="claim"
              size="small"
              disabled={!record.marked_approve || record.approved || record.txState === 'InFlight'}
              onClick={handleClick}
              sx={{ marginLeft: '1em', marginRight: '1em' }}
            >
              {isLoading ? <CircularProgress /> : <DownloadIcon />}
            </Fab>
          </Tooltip>
        )}
      </>
    )
  }

  const URLField = ({ baseURL }: { source: string; baseURL: string }) => {
    const record = useRecordContext<AttestationRequest>()
    let ctype = record.ctype_hash

    if (!ctype.startsWith('kilt:ctype:')) {
      ctype = `kilt:ctype:${ctype}` as ICType['$id']
    }

    return <a href={`${baseURL}${ctype}`}>{ctype}</a>
  }

  return (
    <List>
      <Datagrid expand={ExpandAttestation}>
        <TextField source="id" />
        <TextField source="claimer" />
        <DateField source="created_at" />
        <DateField source="approved_at" />
        <DateField source="revoked_at" />
        <TextField source="tx_state" />
        <URLField source="ctype_hash" baseURL="https://ctypehub.galaniprojects.de/ctype/" />
        {isUserAdmin() && <ApproveButton />}
        {isUserAdmin() && <RevokeButton />}
        {!isUserAdmin() && <DownloadCredential />}
      </Datagrid>
    </List>
  )
}
