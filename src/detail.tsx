import {invoke} from "@tauri-apps/api";
import React, {useEffect, useState} from "react";
import {Col, Row, Skeleton, Image, Button, message, Popconfirm, Descriptions, DescriptionsProps} from "antd";
import {CalcImagePaddleStyle, DateToString} from "./util";
import copy from "copy-to-clipboard";

export default function Detail(props: { imageId: number, jumpIndex: () => void }) {
  const {imageId} = props;
  const [image, setImage] = useState<any>(undefined);
  const [messageApi, contextHolder] = message.useMessage();
  useEffect(() => {
    invoke('get_image', {
      request: {
        limit: 1,
        id: [imageId],
      }
    }).then((value) => {
      const v = value as any;
      setTimeout(() => {
        setImage(v[0]);
      }, 0);
    });
  }, []);
  const ImageCol = () => {
    const width = 180;
    const height = 180;
    if (image === undefined) {
      return <Skeleton.Image active={true} style={{width, height}}/>;
    }
    const style = CalcImagePaddleStyle(width, height, image.width, image.height);
    const src = `data:image/png;base64,${image.image}`;
    return (
      <>
        {contextHolder}
        <Image width={width} height={height} src={src} style={style}/>
        <div style={{marginTop: 10}}>
          <Button type="primary" ghost onClick={() => {
            invoke('re_copy', {image_id: imageId}).then(() => {
              return messageApi.open({
                type: 'success',
                content: '复制成功',
              });
            }).then(() => {
            });
          }}>复制</Button>
          <span style={{marginLeft: 10}}></span>
          <Popconfirm
            title="确认删除这张图片？"
            onConfirm={() => {
              invoke('delete_image', {image_id: imageId}).then(() => {
                messageApi.open({
                  type: 'success',
                  content: '删除成功，一秒后将返回主页。',
                }).then(() => {
                  setTimeout(() => {
                    props.jumpIndex();
                  }, 1000);
                });
              });
            }}
            okText="确认"
            cancelText="取消"
          >
            <Button danger>删除</Button>
          </Popconfirm>
        </div>
      </>
    );
  };
  const DataCol = () => {
    if (image === undefined) {
      return <Skeleton/>;
    }
    const createDate = new Date(image.ctime);
    const modifyDate = new Date(image.mtime);
    const ocr: any = image?.ocr ?? [];
    const text = (() => {
      if (ocr?.code !== 100) {
        return "";
      }
      return ocr?.data?.filter((value: any) => {
        return value?.score > 0.6;
      })?.map((value: any) => {
        return value?.text;
      }).join('\n');
    })();
    return (
      <>
        <Descriptions title="基础信息" column={2} bordered items={[
          {key: '添加时间', label: '添加时间', children: DateToString(createDate)},
          {key: '上次使用', label: '上次使用', children: DateToString(modifyDate)},
          {key: '图片宽度', label: '图片宽度', children: image.width},
          {key: '图片高度', label: '图片高度', children: image.height},
          {key: '数据库ID', label: '数据库ID', children: image.id},
        ]}/>
        {
          text.length > 0 ? (
            <>
              <div style={{marginTop: 20}}></div>
              <Descriptions title="OCR" column={1} items={[{
                key: '1',
                children: <span style={{whiteSpace: "pre-line"}}>{text}</span>,
              }]} extra={<Button onClick={() => {
                copy(text);
                messageApi.open({
                  type: 'success',
                  content: '复制成功',
                }).then(() => {
                });
              }}>复制所有文字</Button>}/>
            </>
          ) : <></>
        }
      </>
    );
  };
  return (
    <Row style={{margin: 10}}>
      <Col span={6}>
        <ImageCol/>
      </Col>
      <Col span={18}>
        <DataCol/>
      </Col>
    </Row>
  );
}