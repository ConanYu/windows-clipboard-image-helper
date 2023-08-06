import {invoke} from "@tauri-apps/api";
import {Button, Image as AntdImage, Input, message, Modal, Popover, Spin} from "antd";
import React, {CSSProperties, useEffect, useState} from "react";
import copy from 'copy-to-clipboard';
import {InView} from "react-intersection-observer";

function CalcImagePaddleStyle(blockWidth: number, blockHeight: number, imageWidth: number, imageHeight: number): CSSProperties {
  if (imageWidth <= 0 || imageHeight <= 0) {
    return {};
  }
  const blockRate = blockWidth / blockHeight;
  const imageRate = imageWidth / imageHeight;
  const paddingUpDown = (() => {
    if (imageRate <= blockRate) {
      return 0;
    }
    return (blockHeight - 1 / imageRate * blockWidth) / 2;
  })();
  const paddingLeftRight = (() => {
    if (imageRate >= blockRate) {
      return 0;
    }
    return (blockWidth - imageRate * blockHeight) / 2;
  })();
  return {
    paddingLeft: paddingLeftRight,
    paddingRight: paddingLeftRight,
    paddingTop: paddingUpDown,
    paddingBottom: paddingUpDown,
  };
}

type PopoverContentProps = {
  imageId: number,
  ctime: number,
  mtime: number,
  copied: boolean,
  setCopied: (clicked: boolean) => void,
  setOpenModal: (openModal: boolean) => void,
};

function PopoverContent(props: PopoverContentProps) {
  const {imageId, ctime, mtime, copied, setCopied, setOpenModal} = props;
  const createDate = new Date(ctime);
  const modifyDate = new Date(mtime);
  const padStart = (x: number) => {
    return x.toString().padStart(2, '0');
  };
  const dateToString = (date: Date) => {
    return `${date.getFullYear()}/${padStart(date.getMonth())}/${padStart(date.getDate())} ${padStart(date.getHours())}:${padStart(date.getMinutes())}:${padStart(date.getSeconds())}`;
  };
  return (
    <div>
      <div>数据库ID：{imageId}</div>
      <div>添加时间：{dateToString(createDate)}</div>
      <div>上次使用：{dateToString(modifyDate)}</div>
      <div style={{marginTop: 3}}>
        <Button onClick={() => {
          setCopied(true);
          invoke('re_copy', {image_id: imageId}).then(() => {
          });
        }} disabled={copied}>
          {copied ? '已' : ''}复制
        </Button>
        <Button style={{marginLeft: 7}} onClick={() => {
          setOpenModal(true);
        }}>显示OCR结果</Button>
      </div>
    </div>
  );
}

function ImageBlock(props: { image: any, text: string, onView?: (inView: boolean, entry: IntersectionObserverEntry) => void }) {
  const {image, text, onView} = props;
  const [messageApi, contextHolder] = message.useMessage();
  const {width, height, ctime, mtime} = image;
  const src = `data:image/png;base64,${image.image}`;
  const blockWidth = 170;
  const blockHeight = 170;
  const style = CalcImagePaddleStyle(blockWidth, blockHeight, width, height);
  const [copied, setCopied] = useState(false);
  const [openModal, setOpenModal] = useState(false);
  const [hover, setHover] = useState(false);
  const modal = openModal ? (
    <>
      {contextHolder}
      <Modal open={openModal} okText="复制全部文字" cancelText="关闭" onOk={() => {
        copy(text);
        messageApi.open({
          type: 'success',
          content: '复制成功',
        }).then(() => {
        });
      }} onCancel={() => setOpenModal(false)} style={{whiteSpace: 'pre-line'}}>{
        text || <div style={{marginTop: 30}}/>
      }</Modal>)
    </>
  ) : <></>;
  return (
    <>
      {onView ? <InView as="span" onChange={onView}></InView> : <></>}
      <Popover open={!openModal && hover}
               content={
                 <PopoverContent imageId={image.id} ctime={ctime} mtime={mtime} copied={copied}
                                 setCopied={setCopied} setOpenModal={(e) => {
                   setOpenModal(e);
                   setHover(false);
                 }}/>
               }
               onOpenChange={(e) => {
                 if (e) {
                   setHover(true);
                 } else {
                   setHover(false);
                   setCopied(false);
                 }
               }}>
        <AntdImage width={blockWidth} height={blockHeight} src={src} style={style}/>
      </Popover>
      {modal}
    </>
  );
}

export default function Index() {
  const [loading, setLoading] = useState(false);
  const [images, setImages] = useState<any[]>([]);
  const [pageNo, setPageNo] = useState(1);
  const [lastImageLen, setLastImageLen] = useState(0);
  const [searchText, setSearchText] = useState('');
  const [searchTextConfirmed, setSearchTextConfirmed] = useState('');
  const showImage = (reload: boolean, showImagePageNo: number, showImageSearchText: string) => {
    setLoading(true);
    if (reload) {
      setImages([]);
    }
    invoke('get_image', {
      request: {
        page_no: showImagePageNo,
        page_size: 16,
        text: !!showImageSearchText ? [showImageSearchText] : undefined,
      }
    }).then((value) => {
      const v = value as any[];
      setLastImageLen(v.length);
      if (reload) {
        setImages(v as any[]);
      } else {
        setImages(images.concat(v));
      }
      setLoading(false);
    });
  };
  useEffect(() => {
    showImage(true, 1, '');
  }, []);
  const content = [];
  let index = 0;
  for (const image of images) {
    if (index % 4 === 0) {
      content.push(<div style={{marginTop: 20}} key={content.length}/>);
      content.push(<span style={{marginLeft: 15}} key={content.length}/>);
    } else {
      content.push(<span style={{marginLeft: 15}} key={content.length}/>);
    }
    const text = (() => {
      const ocr = image.ocr;
      if (ocr.code !== 100) {
        return '';
      }
      let s = [];
      for (const data of ocr.data) {
        s.push(data.text);
      }
      return s.join('\n');
    })();
    content.push(
      <span key={content.length}>
        <ImageBlock image={image} text={text} onView={
          pageNo * 16 - 1 === index ? (inView) => {
            if (inView) {
              setPageNo(pageNo + 1);
              showImage(false, pageNo + 1, searchTextConfirmed);
            }
          } : undefined
        }/>
      </span>
    );
    index += 1;
  }
  const footerStyle: CSSProperties = {
    textAlign: 'center',
    marginTop: 50,
    marginBottom: 30
  };
  return (
    <>
      <div style={{marginTop: 20, marginBottom: 10}}>
        <Input style={{marginLeft: 20, width: 200}} addonBefore="图片文字" onChange={(e) => {
          setSearchText(e.target.value);
        }}/>
        <Button type="primary" style={{marginLeft: 10}} onClick={() => {
          setSearchTextConfirmed(searchText);
          setPageNo(1);
          showImage(true, 1, searchText);
        }}>查询</Button>
      </div>
      {content}
      {loading ? <div style={footerStyle}><Spin size="large"/></div> : <></>}
      {!loading && lastImageLen < 16 ? <div style={footerStyle}>已展示全部内容</div> : <></>}
    </>
  );
}