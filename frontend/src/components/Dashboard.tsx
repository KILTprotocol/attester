import { useEffect, useState } from 'react'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'
import Card from '@mui/material/Card'
import Typography from '@mui/material/Typography'
import { getAxiosClient } from '../api/dataProvider'
import CircularProgress from '@mui/material/CircularProgress'

interface AttestationChartData {
  date: string
  totalAttestationsCreated: number
}

interface KPI {
  attestationsCreatedOverTime: AttestationChartData[]
  attestationsNotApproved: number
  attestationsRevoked: number
  totalClaimers: number
}

const Dashboard = () => {
  const [kpi, setKpi] = useState<KPI>()
  const [errorMessage, setErrorMessage] = useState<string>('')

  useEffect(() => {
    const fetchData = async () => {
      try {
        const client = await getAxiosClient()
        const apiURL = import.meta.env.VITE_SIMPLE_REST_URL
        const res = await client.get(
          apiURL + '/attestation_request/metric/kpis'
        )
        setKpi(res.data)
      } catch (error) {
        console.error('Error fetching data:', error)
        setErrorMessage(`Error fetching data: ${error}`)
      }
    }

    fetchData()
  }, [])

  const transformDate = (data: AttestationChartData[]) =>
    data.map((dataPoint) => ({
      date: new Date(dataPoint.date).toLocaleDateString(),
      total_attestations_created: dataPoint.totalAttestationsCreated,
    }))

  if (errorMessage !== '') {
    return <span style={{ margin: 'auto' }}>{errorMessage}</span>
  }

  if (!kpi) {
    return <CircularProgress style={{ margin: 'auto' }} size={100} />
  }

  return (
    <div
      style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}
    >
      <Card
        style={{
          margin: '1em',
          height: window.innerHeight * 0.4,
          width: window.innerWidth * 0.9,
        }}
      >
        <Typography variant='h5' gutterBottom sx={{ margin: '1em' }}>
          Total Requested Attestations
        </Typography>
        <ResponsiveContainer width='100%' height='80%'>
          <LineChart
            width={500}
            height={300}
            data={transformDate(kpi.attestationsCreatedOverTime)}
            margin={{
              top: 5,
              right: 30,
              left: 20,
              bottom: 5,
            }}
          >
            <CartesianGrid strokeDasharray='3 3' />
            <XAxis dataKey='date' />
            <YAxis />
            <Tooltip />
            <Line
              type='monotone'
              dataKey='total_attestations_created'
              stroke='#82ca9d'
            />
          </LineChart>
        </ResponsiveContainer>
      </Card>
      <div
        style={{
          display: 'flex',
          flexDirection: 'row',
          justifyContent: 'center',
          width: window.innerWidth * 0.9,
        }}
      >
        <Card
          style={{
            height: window.innerHeight * 0.1,
            width: '33%',
            position: 'relative',
          }}
        >
          <Typography variant='h6' gutterBottom sx={{ margin: '1em' }}>
            Total Attestations not Approved
          </Typography>
          <Typography
            sx={{ position: 'absolute', bottom: 0, right: 0, margin: '1em' }}
            variant='subtitle1'
          >
            {kpi.attestationsNotApproved}
          </Typography>
        </Card>
        <Card
          style={{
            marginLeft: '1em',
            marginRight: '1em',
            height: window.innerHeight * 0.1,
            width: '33%',
            position: 'relative',
          }}
        >
          <Typography variant='h6' gutterBottom sx={{ margin: '1em' }}>
            Total Attestations revoked
          </Typography>
          <Typography
            sx={{ position: 'absolute', bottom: 0, right: 0, margin: '1em' }}
            variant='subtitle1'
          >
            {kpi.attestationsRevoked}
          </Typography>
        </Card>
        <Card
          style={{
            height: window.innerHeight * 0.1,
            width: '33%',
            position: 'relative',
          }}
        >
          <Typography variant='h6' gutterBottom sx={{ margin: '1em' }}>
            Total Claimers
          </Typography>
          <Typography
            sx={{ position: 'absolute', bottom: 0, right: 0, margin: '1em' }}
            variant='subtitle1'
          >
            {kpi.totalClaimers}
          </Typography>
        </Card>
      </div>
    </div>
  )
}

export default Dashboard
