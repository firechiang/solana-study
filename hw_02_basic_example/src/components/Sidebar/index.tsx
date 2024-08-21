import React from 'react';
import {PublicKey} from '@solana/web3.js';
import styled from '@emotion/styled';

import {GRAY, REACT_GRAY, PURPLE, WHITE, DARK_GRAY} from '../../constants';
import {hexToRGB} from '../../utils';
import Button from '../Button';
import {ConnectedMethods} from '../../App';

/**
 * 主界面左边的内容
 */

interface Props {
    publicKey?: PublicKey,
    connectedMethods: ConnectedMethods[],
    connect: () => Promise<void>
}

const Sidebar = React.memo((props: Props) => {
    const {publicKey,connectedMethods,connect} = props;
    const phantomLogo = "https://cdn.sanity.io/images/3nm6d03a/production/2c52333718ae2ae5b69e1acf77400c9a0a62cfc5-1632x700.svg";
    return (
        <Main>
            <Body>
                <Link>
                    <img src={phantomLogo} alt="Phantom" width="200"/>
                </Link>
                {
                    publicKey ? (
                        // 有publicKey说明已连接到Phantom钱包
                        <>
                            <div>
                                <Pre>Connected as</Pre>
                                <Badge>{publicKey.toBase58()}</Badge>
                                <Divider/>
                            </div>
                            {
                                connectedMethods.map((method, i) => (
                                    <Button key={`${method.name}-${i}`} onClick={method.onClick}>
                                        {method.name}
                                    </Button>
                                ))
                            }
                        </>
                    ) : (
                        // 没有publicKey说明没有连接Phantom钱包
                        <Button onClick={connect}>Connect to Phantom</Button>
                    )
                }
            </Body>
            <Tag>Solana和Phantom钱包开发基础示例</Tag>
        </Main>
    );
});

export default Sidebar;


const Main = styled.main`
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 20px;
    align-items: center;
    background-color: ${REACT_GRAY};
    > * {
        margin-bottom: 10px;
    }
    @media (max-width: 768px) {
        width: 100%;
        height: auto;
    }
`;

const Body = styled.div`
    display: flex;
    flex-direction: column;
    align-items: center;
    button {
        margin-bottom: 15px;
    }
`;

const Link = styled('a',{
    // @ts-ignore
    href: 'https://phantom.app/',
    target: '_blank',
    rel: 'noopener noreferrer',
})`
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    text-decoration: none;
    margin-bottom: 30px;
    padding: 5px;
    &:focus-visible {
        outline: 2px solid ${hexToRGB(GRAY, 0.5)};
        border-radius: 6px;
    }
`;

const Pre = styled.pre`
    margin-bottom: 5px;
`;

const Badge = styled.div`
    margin: 0;
    padding: 10px;
    width: 100%;
    color: ${PURPLE};
    background-color: ${hexToRGB(PURPLE,0.2)};
    font-size: 14px;
    border-radius: 6px;
    @media (max-width: 400px) {
        width: 280px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    @media (max-width: 320px) {
        width: 220px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    ::selection {
        color: ${WHITE};
        background-color: ${hexToRGB(PURPLE,0.5)};
    }
    ::-moz-selection {
        color: ${WHITE};
        background-color: ${hexToRGB(PURPLE,0.5)};
    }
`;

const Divider = styled.div`
    border: 1px solid ${DARK_GRAY};
    height: 1px;
    margin: 20px 0;
`;

const Tag = styled.p`
    text-align: center;
    color: ${GRAY};
    a {
        color: ${PURPLE};
        text-decoration: none;
        ::selection {
            color: ${WHITE};
            background-color: ${hexToRGB(PURPLE, 0.5)};
        }
        ::-moz-selection {
            color: ${WHITE};
            background-color: ${hexToRGB(PURPLE, 0.5)};
        }
    }
    @media (max-width: 320px) {
        font-size: 14px;
    }
    ::selection {
        color: ${WHITE};
        background-color: ${hexToRGB(PURPLE, 0.5)};
    }
    ::-moz-selection {
        color: ${WHITE};
        background-color: ${hexToRGB(PURPLE, 0.5)};
    }
`;

