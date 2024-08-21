import React from 'react';
import styled from '@emotion/styled';
import {PublicKey} from '@solana/web3.js';

import {TLog} from '../../types';
import {BLACK,GRAY} from '../../constants';

import Button from '../Button';
import Log from './Log';

interface Props {
    publicKey: PublicKey | undefined,
    logs: TLog[],
    clearLogs: () => void
}

/**
 * 使用React.memo生成的子组件只要props的值不改变子组件不会被重新渲染
 */
const Logs = React.memo((props: Props) => {
    const {publicKey,logs,clearLogs} = props;
    return (
        <StyledSection>
            {
                logs.length > 0 ? (
                    // 有日志渲染显示日志
                    <>
                        {
                            logs.map((log,i) => (<Log key={`${log.status}-${log.method}-${i}`} {...log}/>))
                        }
                        <ClearLogsButton onClick={clearLogs}>清空日志</ClearLogsButton>
                    </>
                ) : (
                    // 没有日志
                    <Row>
                        <span>{'>'}</span>
                        <PlaceholderMessage>
                            {
                                publicKey ? (
                                    // 有publicKey说明已连接钱包
                                    <>
                                        { publicKey.toBase58() }
                                        <span role="img" aria-label="Ghost Emoji">✨</span>
                                    </>
                                ) : (
                                    // 没有publicKey说明没有连接钱包
                                    <>
                                        欢迎来到 Phantom 沙盒。连接到您的 Phantom 钱包并试用...{' '}
                                        <span role="img" aria-label="Ghost Emoji">👻</span>
                                    </>
                                )
                            }
                        </PlaceholderMessage>
                    </Row>
                )
            }
        </StyledSection>
    );
});

export default Logs;

const StyledSection = styled.section`
  position: relative;
  flex: 2;
  padding: 20px;
  background-color: ${BLACK};
  overflow: auto;
  font-family: monospace;
`;

const ClearLogsButton = styled(Button)`
  position: absolute;
  top: 20px;
  right: 20px;
  width: 100px;
`;

const PlaceholderMessage = styled.p`
  color: ${GRAY};
`;

const Row = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  span {
    margin-right: 10px;
  }
`;