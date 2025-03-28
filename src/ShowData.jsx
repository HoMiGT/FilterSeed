import * as React from "react";
import PropTypes from "prop-types";
import CircularProgress from "@mui/material/CircularProgress";
import LinearProgress from "@mui/material/LinearProgress";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box"


function LinearProgressWithLabel(props) {
    return (
        <Box sx={{display: 'flex', alignItems: 'center'}}>
            <Box sx={{width: '100%', mr: 1}}>
                <LinearProgress variant="determinate" {...props} />
            </Box>
            <Box sx={{minWidth: 35}}>
                <Typography variant="body2" color="text.secondary">{`${Math.round(
                    props.value,
                )}%`}</Typography>
            </Box>
        </Box>
    );
}

LinearProgressWithLabel.propTypes = {
    value: PropTypes.number.isRequired,
};

function CircularProgressWithLabel(props) {
    return (
        <Box sx={{position: 'relative', display: 'inline-flex'}}>
            <CircularProgress variant="determinate" {...props} />
            <Box
                sx={{
                    top: 0,
                    left: 0,
                    bottom: 0,
                    right: 0,
                    position: 'absolute',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                }}
            >
                <Typography variant="caption"
                            component="div"
                            sx={{color: 'text.secondary'}}>
                    {`${Math.round(props.value)}%`}
                </Typography>
            </Box>
        </Box>
    );
}

CircularProgressWithLabel.prototype = {
    value: PropTypes.number.isRequired,
};

function NavTypography(context, value) {
    return (
        <div style={{
            width: 200,
            display: 'flex',
            flexDirection: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: 12,
            marginRight: 20,
            marginTop: 12
        }}>
            <p>{context}</p>
            <Typography variant="caption" component="div" sx={{color: 'text.secondary'}}>
                <p style={{
                    color: 'red',
                    fondWeight: 'bold',
                    fontSize: 16,
                }}>{value}</p>
            </Typography>
        </div>
    );
}

NavTypography.prototype = {
    context: PropTypes.string.isRequired,
    value: PropTypes.number.isRequired,
}

export default function ShowData(findCount, copiedCount, progress) {
    return (
        <div className={"showData"}>
            {NavTypography("找到种子个数: ", findCount)}
            {NavTypography("拷贝成功个数: ", copiedCount)}
            <LinearProgressWithLabel value={progress}/>
        </div>
    );

}