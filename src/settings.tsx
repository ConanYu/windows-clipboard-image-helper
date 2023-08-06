import {Button, Checkbox, InputNumber, message, Select, Skeleton} from "antd";
import {invoke} from "@tauri-apps/api";
import React, {useEffect, useState} from "react";

const {Option} = Select;

export default function Settings() {
  const [settings, setSettings] = useState<any>(undefined);
  const [autoStart, setAutoStart] = useState<boolean>(false);
  type DatabaseLimitType = 'MB' | 'NUM';
  const [databaseLimitType, setDatabaseLimitType] = useState<DatabaseLimitType>('MB');
  const [databaseLimit, setDatabaseLimit] = useState<number>(1024);
  const [databaseLimitMbValid, setDatabaseLimitMbValid] = useState<boolean>(true);
  const [messageApi, contextHolder] = message.useMessage();
  useEffect(() => {
    invoke('get_settings', {}).then((value: any) => {
      setAutoStart(value.auto_start);
      setDatabaseLimitType(value.database_limit_type);
      setDatabaseLimit(value.database_limit);
      setSettings(value);
    });
  }, []);
  if (settings === undefined) {
    return <Skeleton/>;
  }
  return (
    <div style={{marginLeft: 5}}>
      <div style={{marginTop: 15}}/>
      <Checkbox disabled defaultChecked={settings.auto_start} onChange={(e) => {
        setAutoStart(e.target.checked);
      }}>开机自启（暂不支持开启）</Checkbox>
      <div style={{marginTop: 10}}/>
      <Checkbox disabled defaultChecked={true} onChange={(e) => {
        setAutoStart(e.target.checked);
      }}>后台常驻（暂不支持关闭）</Checkbox>
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
          setDatabaseLimitType(e as DatabaseLimitType);
        }}>
          <Option value="MB">MB</Option>
          <Option value="NUM">个</Option>
        </Select>
      )} defaultValue={databaseLimit} status={databaseLimitMbValid ? '' : 'error'}/>
      <div style={{marginTop: 15}}/>
      {contextHolder}
      <Button type="primary" onClick={() => {
        if (databaseLimitMbValid) {
          invoke('set_settings', {
            settings: {
              auto_start: autoStart,
              database_limit_type: databaseLimitType,
              database_limit: databaseLimit,
            }
          }).then(() => {
            messageApi.open({
              type: 'success',
              content: '保存成功',
            }).then(() => {
            });
          });
        }
      }}>确认</Button>
    </div>
  );
}