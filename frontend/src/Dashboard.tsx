import React, { useEffect, useState } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import Card from "@mui/material/Card"
import Typography from '@mui/material/Typography';
import { getAxiosClient } from "./dataProvider";

interface AttestationChartData {
    date: string;
    total_attestations_created: number;
}

interface KPIInterface {
    attestations_created_over_time: AttestationChartData[];
    attestations_not_approved: number;
    attestations_revoked: number;
    total_claimers: number;
}

const Dashboard = () => {
    const [kpi, setKpi] = useState<KPIInterface>();

    useEffect(() => {
        const fetchData = async () => {
            try {
                const client = await getAxiosClient();
                const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
                const res = await client.get(apiURL + "/attestation_request/metric/kpis");
                setKpi(res.data);
            } catch (error) {
                console.error("Error fetching data:", error);
            }
        };

        fetchData();
    }, []);

    const transformDate = (data: AttestationChartData[]) => (
        data.map((dataPoint) => ({
            date: new Date(dataPoint.date).toLocaleDateString(),
            total_attestations_created: dataPoint.total_attestations_created
        }))
    )

    if (!kpi) {
        return <div />;
    }

    return (
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
            <Card style={{ margin: "1em", height: window.innerHeight * 0.4, width: window.innerWidth * 0.9 }}>
                <Typography variant="h5" gutterBottom sx={{ margin: "1em" }}>
                    Total Requested Attestations
                </Typography>
                <ResponsiveContainer width="100%" height="80%">
                    <LineChart
                        width={500}
                        height={300}
                        data={transformDate(kpi.attestations_created_over_time)}
                        margin={{
                            top: 5,
                            right: 30,
                            left: 20,
                            bottom: 5,
                        }}
                    >
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis dataKey="date" />
                        <YAxis />
                        <Tooltip />
                        <Line type="monotone" dataKey="total_attestations_created" stroke="#82ca9d" />
                    </LineChart>
                </ResponsiveContainer>
            </Card>
            <div style={{ display: "flex", flexDirection: "row", justifyContent: "center", width: window.innerWidth * 0.9 }}>
                <Card style={{ height: window.innerHeight * 0.1, width: "33%", position: "relative" }}>
                    <Typography variant="h6" gutterBottom sx={{ margin: "1em" }}>
                        Total Attestations not Approved
                    </Typography>
                    <Typography sx={{ position: "absolute", bottom: 0, right: 0, margin: "1em" }} variant="subtitle1" >
                        {kpi.attestations_not_approved}
                    </Typography>
                </Card>
                <Card style={{ marginLeft: "1em", marginRight: "1em", height: window.innerHeight * 0.1, width: "33%", position: "relative" }}>
                    <Typography variant="h6" gutterBottom sx={{ margin: "1em" }}>
                        Total Attestations revoked
                    </Typography>
                    <Typography sx={{ position: "absolute", bottom: 0, right: 0, margin: "1em" }} variant="subtitle1" >
                        {kpi.attestations_revoked}
                    </Typography>
                </Card>
                <Card style={{ height: window.innerHeight * 0.1, width: "33%", position: "relative" }}>
                    <Typography variant="h6" gutterBottom sx={{ margin: "1em" }}>
                        Total Claimers
                    </Typography>
                    <Typography sx={{ position: "absolute", bottom: 0, right: 0, margin: "1em" }} variant="subtitle1" >
                        {kpi.total_claimers}
                    </Typography>
                </Card>
            </div>
        </div>
    )
};

export default Dashboard;
