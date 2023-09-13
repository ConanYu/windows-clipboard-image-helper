import {
  Button,
  Checkbox,
  InputNumber,
  message,
  Select,
  Skeleton,
  Upload as AntdUpload,
  UploadProps,
  Spin, Progress,
} from "antd";
import {invoke} from "@tauri-apps/api";
import React, {useEffect, useState} from "react";
import {open} from "@tauri-apps/api/dialog";
import {InboxOutlined} from "@ant-design/icons";

const {Option} = Select;
const {Dragger} = AntdUpload;

function Upload() {
  const [messageApi, contextHolder] = message.useMessage();
  const props: UploadProps = {
    showUploadList: false,
    openFileDialogOnClick: false,
    style: {width: '98%'}
  };
  const onClick = () => {
    invoke('escape_blur', {escape: true}).then(() => {
      open({multiple: true, filters: [{name: '图片', extensions: ['png', 'jpg', 'jpeg']}]})
        .then(async (file) => {
          if (file) {
            let imagePath = [];
            if (typeof file === 'string') {
              imagePath.push(file);
            } else {
              imagePath = file;
            }
            return await invoke('upload_image', {image_path: imagePath});
          }
          return 'canceled';
        }).finally(() => {
        invoke('escape_blur', {escape: false}).catch((s: string) => {
          return messageApi.open({
            type: 'error',
            content: s,
          });
        });
      }).then((value) => {
        if (value !== 'canceled') {
          return messageApi.open({
            type: 'success',
            content: '已触发后台上传',
          });
        }
      }).catch((s: string) => {
        return messageApi.open({
          type: 'error',
          content: s,
        });
      });
    }).catch((s: string) => {
      return messageApi.open({
        type: 'error',
        content: s,
      });
    });
  };
  return (
    <>
      {contextHolder}
      <div onClick={onClick}>
        <Dragger {...props}>
          <p className="ant-upload-drag-icon"><InboxOutlined/></p>
          <p className="ant-upload-text">点击此区域即可上传图片</p>
          <p
            className="ant-upload-hint">只能上传jpg格式或png格式的图片，上传的图片仅在本地保存和分析，上传速度可能比较慢。</p>
        </Dragger>
      </div>
    </>
  )
}

export default function Settings() {
  const [ready, setReady] = useState(false);
  const [autoStart, setAutoStart] = useState<boolean>(false);
  const [databaseLimitType, setDatabaseLimitType] = useState<'MB' | 'NUM'>('MB');
  const [databaseLimit, setDatabaseLimit] = useState<number>(1024);
  const [databaseLimitMbValid, setDatabaseLimitMbValid] = useState<boolean>(true);
  const [ocrStatus, setOcrStatus] = useState<number | undefined>(undefined);
  const [ocrDownloading, setOcrDownloading] = useState<boolean>(false);
  const [ocrFeature, setOcrFeature] = useState<boolean>(false);
  const [messageApi, contextHolder] = message.useMessage();
  useEffect(() => {
    invoke('ocr_status', {}).then((value) => {
      const v = value as number;
      if (v < 0.0) {
        setOcrDownloading(false);
        setOcrStatus(v + 111.1);
      } else {
        setOcrDownloading(true);
        setOcrStatus(v);
      }
    }).catch((e) => {
      console.error(e);
    });
  }, []);
  if (!ready) {
    invoke('get_settings', {}).then((value: any) => {
      setAutoStart(value.auto_start);
      setDatabaseLimitType(value.database_limit_type);
      setDatabaseLimit(value.database_limit);
      setOcrFeature(value.ocr_feature);
      setReady(true);
    });
    return <Skeleton style={{marginLeft: 15, marginTop: 15, width: '96%'}}/>;
  }
  const Header = (props: { text: string }) => {
    return <h3>{props.text}</h3>;
  };
  const OCRContent = () => {
    if (ocrStatus === undefined) {
      return <Spin/>;
    }
    if (ocrStatus > 100.0) {
      return <div>状态：<span style={{color: '#00AA00'}}>可用</span></div>;
    }
    if (ocrDownloading) {
      setTimeout(() => {
        invoke('ocr_status', {}).then((value) => {
          setOcrStatus(value as number + Math.random() / 1e9);
        }).catch((e) => {
          console.error(e);
        });
      }, 2000);
    }
    return (
      <>
        <div>状态：
          {
            ocrDownloading ? (
              <>
                <span style={{color: '#0000AA'}}>下载中</span>
                <Button style={{marginLeft: 10}} onClick={() => {
                  invoke('ocr_pause_prepare', {}).then(() => {
                    setOcrDownloading(false);
                  }).catch((e) => {
                    console.error(e);
                  });
                }}>暂停下载</Button>
              </>
            ) : (
              <>
                <span style={{color: '#AA0000'}}>不可用</span>
                <Button style={{marginLeft: 10}} onClick={() => {
                  invoke('ocr_prepare', {}).then(() => {
                    setOcrDownloading(true);
                  }).catch((e) => {
                    console.error(e);
                  });
                }}>下载插件</Button>
              </>
            )
          }
        </div>
        {
          ocrDownloading ? (
            <div style={{width: '95%', marginTop: 10}}>
              <Progress percent={Math.round(ocrStatus * 100.0) / 100.0} status="active"/>
            </div>
          ) : <></>
        }
      </>
    );
  };
  return (
    <div style={{marginLeft: 15}}>
      <div style={{marginTop: 15}}/>
      <Header text="设置"/>
      <Checkbox checked={autoStart} onChange={(e) => {
        setAutoStart(e.target.checked);
      }}>开机自启</Checkbox>
      <div style={{marginTop: 10}}/>
      <InputNumber addonBefore="数据库存储上限" style={{width: 333}} onChange={(e) => {
        const x = Number(e);
        if (isNaN(x) || x <= 0) {
          setDatabaseLimitMbValid(false);
        } else {
          setDatabaseLimitMbValid(true);
          setDatabaseLimit(e as number);
        }
      }} addonAfter={(
        <Select defaultValue={databaseLimitType} style={{width: 100}} onChange={(e) => {
          setDatabaseLimit(1024);
          setDatabaseLimitType(e);
        }}>
          <Option value="MB">MB</Option>
          <Option value="NUM">个</Option>
        </Select>
      )} defaultValue={databaseLimit} status={databaseLimitMbValid ? '' : 'error'}/>
      <div style={{marginTop: 15}}/>
      <Checkbox disabled={!ocrStatus || ocrStatus < 100.0} onChange={(e) => {
        setOcrFeature(e.target.checked);
      }} checked={ocrFeature}>OCR功能</Checkbox>
      <div style={{marginTop: 15}}/>
      {contextHolder}
      <Button type="primary" onClick={() => {
        if (databaseLimitMbValid) {
          invoke('set_settings', {
            settings: {
              auto_start: autoStart,
              database_limit_type: databaseLimitType,
              database_limit: databaseLimit,
              ocr_feature: ocrFeature,
            }
          }).then(() => {
            messageApi.open({
              type: 'success',
              content: '保存成功',
            }).then(() => {
            });
          }).catch((msg: string) => {
            messageApi.open({
              type: 'error',
              content: msg,
            }).then(() => {
            });
          });
        }
      }}>确认</Button>
      <h4>OCR</h4>
      <div><OCRContent/></div>
      <Header text="上传图片"/>
      <Upload/>
    </div>
  );
}