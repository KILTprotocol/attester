import { Create, SaveButton, SimpleForm, Toolbar, useNotify } from 'react-admin'
import TextField from '@mui/material/TextField'
import { ICType, IClaimContents, Claim, DidUri, Credential as KiltCredential } from '@kiltprotocol/sdk-js'
import { useState } from 'react'
import ReactJson, { InteractionProps } from 'react-json-view'

import { fetchCType } from '../utils/utils'

function getFormattedDate(): string {
  const date = new Date()
  const day = date.getDate().toString().padStart(2, '0')
  const month = (date.getMonth() + 1).toString().padStart(2, '0')
  const year = date.getFullYear().toString()
  return `${year}-${month}-${day}`
}

//TODO:fix "It currently only works with ctypes with a height of 1."
function getDefaultEntryForType({ type, format }: { type: string; format?: string }) {
  if (format === 'date' && type === 'string') {
    return getFormattedDate()
  }
  if (type === 'string') {
    return ''
  }
  if (type === 'boolean') {
    return false
  }
  if (type === 'number' || type === 'integer') {
    return 0
  }
}

export default function AttestationCreate() {
  // states
  const [ctypeHash, setCtypeHash] = useState<string>('')
  const [claimer, setClaimer] = useState<string>('')
  const [ctype, setCtypeDetails] = useState<ICType>()

  const [claimContent, setClaimContent] = useState<IClaimContents>()

  // hooks
  const notify = useNotify()

  //callbacks
  const handleSelectedCtype = async (ctype: string) => {
    const fmtCtype = ctype.trim()
    setCtypeHash(fmtCtype)
    try {
      const ctypeDetails = await fetchCType(fmtCtype as unknown as `kilt:ctype:0x${string}`)
      const claimContent: Record<string, any> = {}
      Object.entries(ctypeDetails.cType.properties).map(
        ([key, type]) => (claimContent[key] = getDefaultEntryForType(type as { type: string; format?: string })),
      )
      setCtypeDetails(ctypeDetails.cType)
      setClaimContent(claimContent)
    } catch {
      setClaimContent(undefined)
      notify('CType does not exists', { type: 'error' })
    }
  }

  const onEdit = (data: InteractionProps) => {
    const { existing_value, name, updated_src, new_value } = data

    let fmtSrc

    if (Number.isInteger(existing_value) && new_value && name) {
      fmtSrc = {
        ...updated_src,
        [name]: +new_value,
      }
    }

    setClaimContent(fmtSrc ? fmtSrc : (data.updated_src as IClaimContents))
  }

  const transformData = () => {
    if (!ctype || !claimContent) {
      return undefined
    }

    try {
      const claim = Claim.fromCTypeAndClaimContents(ctype, claimContent, claimer as DidUri)
      return KiltCredential.fromClaim(claim)
    } catch (e) {
      console.error(e)
      notify('Ctype Verification failed', { type: 'error' })

      throw e
    }
  }

  //Elements
  const CustomToolBar = () => {
    return (
      <Toolbar>
        <SaveButton alwaysEnable={claimer !== '' && claimContent !== undefined} label="Save" type="button" />
      </Toolbar>
    )
  }

  return (
    <Create transform={transformData} redirect="list">
      <SimpleForm toolbar={<CustomToolBar />}>
        <TextField
          value={ctypeHash}
          label="Ctype Hash"
          variant="outlined"
          fullWidth
          onChange={(e) => handleSelectedCtype(e.target.value)}
          required
        />
        <TextField
          value={claimer}
          label="Claimer"
          variant="outlined"
          fullWidth
          onChange={(e) => setClaimer(e.target.value)}
          required
        />
        {claimContent && (
          <ReactJson src={claimContent} onEdit={onEdit} name="Claim" validationMessage="Claim Verification failed" />
        )}
      </SimpleForm>
    </Create>
  )
}
