import { useEffect, useState } from "react";
import CircularProgress from "@mui/material/CircularProgress";
import { Line } from 'react-chartjs-2';

import { getAxiosClient } from "./dataProvider";

interface AttestationData {
    date: string;
    total_attestations_created: number;
}

interface KPIInterface {
    attestations_created_over_time: AttestationData[];
    attestations_not_approved: number;
    attestations_revoked: number;
    total_claimers: number;
}


interface LineChartProps {
    data: AttestationData[];
}

const LineChart: React.FC<LineChartProps> = ({ data }) => {
    const dates = data.map((entry) => entry.date);
    const totalAtt = data.map((entry) => entry.total_attestations_created);

    const chartData = {
        labels: dates,
        datasets: [
            {
                label: 'Total Attestations Created',
                data: totalAtt,
                fill: false,
                borderColor: 'blue',
            },
        ],
    };

    return (
        <div>
            <Line data={chartData} />
        </div>
    );
};



export const Dashboard = () => {
    let [kpi, setKpi] = useState<KPIInterface>();
    useEffect(() => {
        getAxiosClient().then((client) => {
            const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
            client.get(apiURL + "/attestation_request/metric/kpis").then((res) => {
                setKpi(res.data)

            })
        });

    }, [])

    if (!kpi) {
        return <CircularProgress />
    }

    console.log(kpi)


    return (
        <div>
        </div>
    )


};



