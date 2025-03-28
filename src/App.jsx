import * as React from "react";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";
import ShowData from "./ShowData.jsx";
import ChoicePath from "./ChoicePath.jsx";
import Button from '@mui/material/Button';
import NotStartedOutlinedIcon from '@mui/icons-material/NotStartedOutlined';


function App() {
    const [isDisabled, setIsDisabled] = React.useState(false);
    const [findCount, setFindCount] = React.useState(0);
    const [copiedCount, setCopiedCount] = React.useState(0);
    const [progress, setProgress] = React.useState(0);

    const PathsRef = React.useRef(null);
    const timerRef = React.useRef(null);

    React.useEffect(() => {
        if (isDisabled) {
            timerStart();
        } else {
            timerStop();
        }
    }, [isDisabled]);

    function timerStart() {
        timerRef.current = setInterval(async () => {
            if (isDisabled) {
                await invoke("copy_process").then((result) => {
                    setFindCount(result.total);
                    setCopiedCount(result.copied);
                    setProgress(result.rate);
                }).catch((e) => {
                    console.error("copy_process error: ", e);
                });
            }
        }, 1000);
    }

    function timerStop() {
        clearInterval(timerRef.current);
    }

    async function handleStartCopy() {
        try {
            setFindCount(0);
            setCopiedCount(0);
            setProgress(0);
            setIsDisabled(true);
            const choice_file = PathsRef.current.filePath;
            const seed_dir = PathsRef.current.srcDirPath;
            const target_dir = PathsRef.current.dstDirPath;
            await invoke("start_copy", {
                choiceFile: choice_file,
                seedDir: seed_dir,
                targetDir: target_dir
            }).catch((e) => {
                setIsDisabled(false);
                console.error("start_copy error: ", e);
            }).then((_) => {
                setIsDisabled(false);
            });
        } catch (e) {
            setIsDisabled(false);
            console.error("handleStartCopy error: ", e);
        }
    }

    return (
        <main className="container">
            <div className="box" style={{
                alignContent: "center",
                textAlign: "center",
                width: 614,
            }}>
                <div style={{
                    alignContent: "center",
                    textAlign: "center",
                    display: "flex",
                    flexDirection: "row",
                }}>
                    <ChoicePath ref={PathsRef}/>
                    {ShowData(findCount, copiedCount, progress)}
                </div>
                <hr style={{
                    border: "0",
                    height: "1px",
                    background: "linear-gradient(to right, #ccc, #666, #ccc)",
                    margin: "20px 0",
                }}/>
                <div style={{
                    alignContent: "Right",
                    textAlign: "end",
                }}>
                    <Button variant="contained" id={"start-copy"}
                            disabled={isDisabled}
                            onClick={async () => {
                                await handleStartCopy();
                            }}
                            endIcon={<NotStartedOutlinedIcon/>}>开始拷贝</Button>
                </div>
            </div>
        </main>
    );
}

export default App;
