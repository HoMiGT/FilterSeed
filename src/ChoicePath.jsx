import * as React from 'react';
import {forwardRef, useImperativeHandle} from "react";
import Stack from '@mui/material/Stack';
import IconButton from "@mui/material/IconButton";
import DescriptionOutlinedIcon from '@mui/icons-material/DescriptionOutlined';
import FolderOpenRoundedIcon from '@mui/icons-material/FolderOpenRounded';
import Input from '@mui/material/Input';
import {invoke} from "@tauri-apps/api/core";

function NavInput({text, input_id, value, setValue}) {
    return (
        <Stack spacing={1} sx={{width: 300}}>
            <Input onChange={(e) => {
                setValue(e.target.value);
            }} placeholder={text} id={input_id} value={value}
            />
        </Stack>
    );
}

function NavFileButton({callback}) {
    return (
        <IconButton size="large" onClick={async () => {
            callback();
        }}>
            <DescriptionOutlinedIcon/>
        </IconButton>
    );
}

function NavDirButton({callback}) {
    return (
        <IconButton size="large" onClick={async () => {
            callback();
        }}>
            <FolderOpenRoundedIcon/>
        </IconButton>
    );
}

function ChoicePath(props, ref) {
    const [filePath, setFilePath] = React.useState("");
    const [srcDirPath, setSrcDirPath] = React.useState("");
    const [dstDirPath, setDstDirPath] = React.useState("");

    useImperativeHandle(ref, () => ({
        filePath,
        srcDirPath,
        dstDirPath,
    }));

    async function handleFileClick() {
        await invoke("choice_file").catch((e) => {
            console.error("choice_file error: ", e);
        }).then((result) => {
            setFilePath(result);
        });
    }

    async function handleSrcDirClick() {
        await invoke("choice_src_dir").catch((e) => {
            console.error("choice_src_dir: ", e);
        }).then((result) => {
            setSrcDirPath(result);
        });
    }

    async function handleDstDirClick() {
        await invoke("choice_dst_dir").catch((e) => {
            console.error("choice_dst_dir error: ", e);
        }).then((result) => {
            setDstDirPath(result);
        });
    }

    return (
        <div className="choicePath" style={{
            display: "flex",
            flexDirection: "column",
        }}>
            <div style={{
                display: "inline-flex",
                flexDirection: "row",
            }}>
                <div>
                    <NavInput text={"请选择文件"} input_id={"file_path"} value={filePath} setValue={setFilePath}/>
                </div>
                <div style={{
                    marginLeft: 10
                }}>
                    {NavFileButton({callback: handleFileClick})}
                </div>

            </div>
            <div style={{
                display: "inline-flex",
                flexDirection: "row",
            }}>
                <div>
                    <NavInput text={"请选择种子目录"} input_id={"file_path"} value={srcDirPath}
                              setValue={setSrcDirPath}/>
                </div>
                <div style={{
                    marginLeft: 10
                }}>
                    {NavDirButton({callback: handleSrcDirClick})}
                </div>
            </div>
            <div style={{
                display: "inline-flex",
                flexDirection: "row",
            }}>
                <div>
                    <NavInput text={"请选择保存目录"} input_id={"file_path"} value={dstDirPath}
                              setValue={setDstDirPath}/>
                </div>
                <div style={{
                    marginLeft: 10
                }}>
                    {NavDirButton({callback: handleDstDirClick})}
                </div>
            </div>
        </div>
    );
}

export default forwardRef(ChoicePath);


