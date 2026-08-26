use std::io::BufRead;

pub struct XmlDocument;

enum State {
    Prologue,
    Finished,
}

impl XmlDocument {
    pub fn parse(xml: impl BufRead) -> Result<XmlDocument, String> {
        let mut state = State::Prologue;

        #[loop_match]
        'machine: loop {
            state = 'state: {match state {
                State::Prologue => {
                    break 'state State::Finished;
                },

                State::Finished => {
                    break 'machine;
                }
            }}
        }

        todo!()
    }
}
