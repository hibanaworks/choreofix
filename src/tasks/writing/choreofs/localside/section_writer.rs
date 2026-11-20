//! Actual requests, both owned section views, effects and original results.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::PairedEffects};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run<'run, const ROLE: u8, const FIRST: u8, const SECOND: u8>(
    endpoint: &mut Endpoint<'run, ROLE>,
    effects: &impl PairedEffects,
) -> ShellResult<()> {
    let input = imp::assignment(ROLE);
    let arguments = endpoint.recv::<ReadRequest<ROLE>>().await?;
    let read = effects.apply(arguments).await?;
    imp::result(&read, "read_literature", input.agent)?;
    if read["success"] != true || !read["data"]["artifact"]["id"].is_string() {
        return Err(ShellError::message(
            "literature effect did not return its actual input",
        ));
    }
    let bytes = imp::encode(&read)?;
    endpoint.send::<ReadResult>(&bytes.as_slice()).await?;

    let arguments = endpoint.recv::<DraftRequest<ROLE>>().await?;
    let draft = effects.apply(arguments).await?;
    imp::result(&draft, "draft_content", input.agent)?;
    if draft["success"] != true
        || draft["data"]["artifact"]["literature"][input.topic] != read["data"]["artifact"]
    {
        return Err(ShellError::message(
            "draft did not consume its actual literature",
        ));
    }
    let bytes = imp::encode(&draft)?;
    endpoint.send::<DraftResult>(&bytes.as_slice()).await?;

    let admitted = endpoint.recv::<WriteRequest<ROLE>>().await?;
    let arguments = &admitted.arguments;
    let (bytes, written) = {
        let [mut a, mut b] = endpoint.par()?;
        let ((mut first, request_a), (mut second, request_b)) = async {
            async {
                let mut first = a.recv::<Loan<FIRST>>().await?;
                let request = async {
                    async {
                        first.send::<SectionRequest<FIRST, ROLE>>(arguments).await?;
                    };
                    Ok(a.recv::<SectionRequest<FIRST, ROLE>>().await?)
                }
                .await?;
                Ok((first, request))
            };
            async {
                let mut second = b.recv::<Loan<SECOND>>().await?;
                let request = async {
                    async {
                        second
                            .send::<SectionRequest<SECOND, ROLE>>(arguments)
                            .await?;
                    };
                    Ok(b.recv::<SectionRequest<SECOND, ROLE>>().await?)
                }
                .await?;
                Ok((second, request))
            };
        }
        .await?;
        let admitted = admitted.value();
        if request_a.as_bytes() != request_b.as_bytes()
            || crate::imp::json::decode(request_a.as_bytes())? != admitted
        {
            return Err(ShellError::message(
                "section requests differ from the actual file request",
            ));
        }
        let written = effects.apply_pair(request_a, request_b).await?;
        imp::result(&written, "write_to_document", input.agent)?;
        if written["success"] != true
            || written["data"]["artifact"]["draft"] != draft["data"]["artifact"]
        {
            return Err(ShellError::message(
                "document effect did not consume its actual draft",
            ));
        }
        let bytes = imp::encode(&written)?;
        async {
            async {
                async {
                    async {
                        a.send::<SectionWritten<FIRST>>(&bytes.as_slice()).await?;
                    };
                    let received = first.recv::<SectionWritten<FIRST>>().await?;
                    if received[..] != bytes {
                        return Err(ShellError::message("first section result differs"));
                    }
                    Ok(())
                }
                .await?;
                a.send::<Loan<FIRST>>(first).await?;
            };
            async {
                async {
                    async {
                        b.send::<SectionWritten<SECOND>>(&bytes.as_slice()).await?;
                    };
                    let received = second.recv::<SectionWritten<SECOND>>().await?;
                    if received[..] != bytes {
                        return Err(ShellError::message("second section result differs"));
                    }
                    Ok(())
                }
                .await?;
                b.send::<Loan<SECOND>>(second).await?;
            };
            Ok(())
        }
        .await?;
        (bytes, written)
    };
    endpoint.send::<WriteResult>(&bytes.as_slice()).await?;

    let arguments = endpoint.recv::<ReviewRequest<ROLE>>().await?;
    let reviewed = effects.apply(arguments).await?;
    imp::result(&reviewed, "review_own_work", input.agent)?;
    if reviewed["data"]["input_snapshot"] != written["data"]["artifact"] {
        return Err(ShellError::message(
            "review did not examine the actual document",
        ));
    }
    let bytes = imp::encode(&reviewed)?;
    endpoint.send::<ReviewResult>(&bytes.as_slice()).await?;
    Ok(())
}
